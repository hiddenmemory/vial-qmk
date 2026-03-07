use alloc::{boxed::Box, format, rc::Rc, vec::Vec};
use serde::{Serialize, de::DeserializeOwned};

use crate::{heap, utils::debug_log};

use hid_bridge::{Empty, MessageHeader, MessageType, QMK_RS_CHANNEL, QMK_RS_CHANNEL_LENGTH};

type Bridge = Box<dyn Fn(&mut MessageHeader, &mut [u8]) -> bool>;

static mut USB_HANDLERS: Option<Vec<(MessageType, Option<Bridge>)>> = None;

pub fn initialise() {
    unsafe {
        USB_HANDLERS = Some(Vec::with_capacity(QMK_RS_CHANNEL_LENGTH));
    }

    hid_listen::<Empty, Empty, _>(MessageType::Ping, |_, _| (Some(MessageType::Pong), None));

    hid_listen::<Empty, Empty, _>(MessageType::HeapUsage, |_, _| {
        let (total, used, free) = heap::usage();

        debug_log(&format!(
            "request for memory usage: total={total}, used={used}, free={free} ({}%)",
            heap::usage_percentage()
        ));

        (None, None)
    });
}

pub fn hid_listen<
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

                    raw_body[5..buffer.len() + 5].copy_from_slice(&buffer[..]);
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
    for (potential, handler) in handlers.iter_mut() {
        if message_type.eq(potential) {
            let _ = handler.take();
        }
    }

    handlers.push((message_type, Some(outer_f)));
}

fn check(data: &mut [u8]) -> bool {
    if data[0] != QMK_RS_CHANNEL {
        debug_log(&format!(
            "[hid] expected {QMK_RS_CHANNEL} first byte magic, got {}",
            data[0]
        ));
        return false;
    }

    let (header, body) = data.split_at_mut(5);

    let mut unpacked_header = match postcard::from_bytes::<MessageHeader>(header) {
        Ok(value) => value,
        Err(err) => {
            debug_log(&format!("[hid] unable to decode header: {err}"));
            return false;
        }
    };

    debug_log(&format!(
        "[hid] message_type: {:?}, length: {}, packet {} of {}",
        unpacked_header.message_type,
        unpacked_header.packet_length,
        unpacked_header._reserved_packet_counter,
        unpacked_header._reserved_packet_total
    ));

    let handlers = unsafe {
        #[allow(static_mut_refs)]
        USB_HANDLERS.as_mut().unwrap()
    };

    for (potential, handler) in handlers.iter_mut() {
        if unpacked_header.message_type.ne(potential) {
            continue;
        }

        if let Some(handler) = handler
            && handler(&mut unpacked_header, body)
        {
            let _ = postcard::to_slice(&unpacked_header, header);
            return true;
        }
    }

    debug_log(&format!(
        "[hid] unhandled message: {:?}",
        unpacked_header.message_type
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
                "unhandled hid packet: {:X},{:X}",
                actual_data[0], actual_data[1]
            ));
        }
    }
}
