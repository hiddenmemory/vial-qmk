use alloc::{boxed::Box, format, rc::Rc, vec::Vec};
use serde::{Serialize, de::DeserializeOwned};

use hid_bridge::{
    MessageHeader, MessageType, QMK_RS_CHANNEL, QMK_RS_CHANNEL_LENGTH, QMK_RS_HEADER_LENGTH,
};

use crate::{
    keyboard::{Channel, Keyboard},
    utils::debug::debug_log,
};

type Bridge = Box<dyn Fn(&mut MessageHeader, &mut [u8]) -> bool>;

static mut USB_HANDLERS: Option<Vec<(MessageType, Option<Bridge>, bool)>> = None;

mod listeners;

pub fn initialise() {
    unsafe {
        USB_HANDLERS = Some(Vec::with_capacity(QMK_RS_CHANNEL_LENGTH));
    }

    listeners::listen_for_ping();
    listeners::listen_for_toggle_debug();
    listeners::listen_for_set_frame_time();
    listeners::listen_for_heap_usage();
    listeners::listen_for_wake();
    listeners::listen_for_display_brightness();
    listeners::listen_for_date_time();

    if Keyboard::is_secondary() {
        unsafe {
            qmk_sys::transaction_register_rpc(
                Channel::UsbForward.to_qmk_id(),
                Some(usb_forward_bridge),
            );
        }

        debug_log("[hid] initialised for secondary");
    } else {
        debug_log("[hid] initialised for primary");
    }
}

pub fn listen<
    IncomingBodyType: DeserializeOwned + 'static,
    OutgoingBodyType: Serialize + DeserializeOwned + 'static,
    F,
>(
    message_type: MessageType,
    f: F,
) where
    F: Fn(
            &MessageHeader,
            Option<IncomingBodyType>,
        ) -> (Option<MessageType>, Option<OutgoingBodyType>)
        + 'static,
{
    listen_and_forward(message_type, false, f)
}

pub fn listen_and_forward<
    IncomingBodyType: DeserializeOwned + 'static,
    OutgoingBodyType: Serialize + DeserializeOwned + 'static,
    F,
>(
    message_type: MessageType,
    forward: bool,
    f: F,
) where
    F: Fn(
            &MessageHeader,
            Option<IncomingBodyType>,
        ) -> (Option<MessageType>, Option<OutgoingBodyType>)
        + 'static,
{
    // If we are the secondary keyboard, and we are not a forward, we can ignore this listen request
    if Keyboard::is_secondary() {
        if forward {
            debug_log(&format!("[usb] listening for forward on {message_type:?}"));
        } else {
            return;
        }
    }

    let inner_f = Rc::new(Box::new(f));

    let outer_f: Bridge = Box::new(move |header: &mut MessageHeader, raw_body: &mut [u8]| {
        let optional_body: Option<IncomingBodyType> = if header.packet_length == 0 {
            None
        } else if header.packet_length > 27 {
            debug_log(&format!(
                "[hid][{:?}] incoming packet body length is > 27 which can't be true",
                header.message_type
            ));
            return false;
        } else {
            match postcard::from_bytes::<IncomingBodyType>(raw_body) {
                Ok(value) => Some(value),
                Err(err) => {
                    debug_log(&format!(
                        "[hid] unable to decode incoming packet body: {err}"
                    ));
                    return false;
                }
            }
        };

        let (optional_type_rewrite, optional_body_rewrite) =
            inner_f.as_ref()(header, optional_body);

        if let Some(value) = optional_type_rewrite {
            header.message_type = value;
        }

        if let Some(value) = optional_body_rewrite {
            match postcard::to_allocvec(&value) {
                Ok(buffer) => {
                    if buffer.len() > 27 {
                        debug_log(&format!(
                            "[hid][{:?}] outgoing packet body length is > 27 which can't be true",
                            header.message_type
                        ));
                        return false;
                    }

                    header.packet_length = buffer.len() as u8;

                    raw_body[QMK_RS_HEADER_LENGTH..buffer.len() + QMK_RS_HEADER_LENGTH]
                        .copy_from_slice(&buffer[..]);
                }
                Err(err) => {
                    debug_log(&format!(
                        "[hid] unable to decode outgoing packet body: {err}"
                    ));
                    return false;
                }
            }
        }

        true
    });

    let handlers = unsafe {
        #[allow(static_mut_refs)]
        USB_HANDLERS.as_mut().unwrap()
    };

    // This will disable existing channels with the same message id
    // it is unlikely that there will be repeated registrations. If
    // there does become that, the handlers array can be filtered
    for (potential, handler, _) in handlers.iter_mut() {
        if message_type.eq(potential) {
            let _ = handler.take();
        }
    }

    handlers.push((
        message_type,
        Some(outer_f),
        if Keyboard::is_primary() {
            forward
        } else {
            false
        },
    ));
}

fn check(data: &mut [u8]) -> bool {
    let tag = if Keyboard::is_primary() {
        "hid:raw"
    } else {
        "hid:forward"
    };

    if data[0] != QMK_RS_CHANNEL {
        debug_log(&format!(
            "[{tag}] expected {QMK_RS_CHANNEL} first byte magic, got {}",
            data[0]
        ));
        return false;
    }

    let mut header = match postcard::from_bytes::<MessageHeader>(data) {
        Ok(value) => value,
        Err(err) => {
            debug_log(&format!("[{tag}] unable to decode header: {err}"));
            return false;
        }
    };

    debug_log(&format!(
        "[{tag}] message_type: {:?}, length: {}, packet {} of {}",
        header.message_type,
        header.packet_length,
        header._reserved_packet_counter,
        header._reserved_packet_total
    ));

    let handlers = unsafe {
        #[allow(static_mut_refs)]
        USB_HANDLERS.as_mut().unwrap()
    };

    for (potential, handler, forward) in handlers.iter_mut() {
        if header.message_type.ne(potential) {
            continue;
        }

        if let Some(handler) = handler {
            if *forward {
                let result = unsafe {
                    qmk_sys::transaction_rpc_exec(
                        Channel::UsbForward.to_qmk_id(),
                        data.len() as u8,
                        data.as_ptr() as *const core::ffi::c_void,
                        0,
                        core::ptr::null_mut(),
                    )
                };

                if !result {
                    debug_log(&format!(
                        "[{tag}] unable to forward {:?} to secondary",
                        potential
                    ));
                }
            }

            let (raw_header, raw_body) = data.split_at_mut(QMK_RS_HEADER_LENGTH);

            if handler(&mut header, raw_body) {
                let _ = postcard::to_slice(&header, raw_header);
            }

            return true;
        }
    }

    debug_log(&format!(
        "[{tag}] unhandled message: {:?}",
        header.message_type
    ));

    false
}

#[unsafe(no_mangle)]
/// # Safety
///
/// This is safe
pub unsafe extern "C" fn raw_hid_receive_rs(data: *mut u8, length: u8) {
    unsafe {
        let actual_data = alloc::slice::from_raw_parts_mut(data, length as usize);

        if !check(actual_data) {
            debug_log(&format!(
                "[hid:raw] unhandled hid packet: {:X},{:X}",
                actual_data[0], actual_data[1]
            ));
        }
    }
}

unsafe extern "C" fn usb_forward_bridge(
    in_len: u8,
    in_data: *const core::ffi::c_void,
    _out_len: u8,
    _out_data: *mut core::ffi::c_void,
) {
    #[allow(static_mut_refs)]
    unsafe {
        let incoming_data = alloc::slice::from_raw_parts(in_data as *const u8, in_len as usize);

        let mut forward_data = Vec::with_capacity(incoming_data.len());
        forward_data.extend_from_slice(incoming_data);

        if !check(&mut forward_data) {
            debug_log(&format!(
                "[hid:forward] unhandled hid packet: {:X},{:X}",
                incoming_data[0], incoming_data[1]
            ));
        }
    }
}
