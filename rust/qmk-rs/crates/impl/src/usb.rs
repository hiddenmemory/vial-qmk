use alloc::{boxed::Box, format, rc::Rc, vec::Vec};
use serde::{Serialize, de::DeserializeOwned};

use crate::{heap, utils::debug_log};

use hid_bridge::{MessageHeader, MessageType, QMK_RS_CHANNEL, QMK_RS_CHANNEL_LENGTH};

type Bridge = Box<dyn Fn(&mut MessageHeader, &mut [u8]) -> bool>;

static mut HANDLERS: Option<Vec<(MessageType, Option<Bridge>)>> = None;

pub fn initialise() {
    unsafe {
        HANDLERS = Some(Vec::with_capacity(QMK_RS_CHANNEL_LENGTH));
    }

    listen::<hid_bridge::Empty, hid_bridge::Empty, _>(MessageType::Ping, |_, _| {
        (Some(MessageType::Pong), None)
    });

    listen::<hid_bridge::Empty, hid_bridge::Empty, _>(MessageType::HeapUsage, |_, _| {
        let (total, used, free) = heap::usage();

        debug_log(&format!(
            "request for memory usage: total={total}, used={used}, free={free} ({}%)",
            heap::usage_percentage()
        ));

        (None, None)
    });
}

pub fn check(data: &mut [u8]) -> bool {
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
        HANDLERS.as_mut().unwrap()
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

pub fn listen<
    RequestType: DeserializeOwned + 'static,
    ResponseType: Serialize + DeserializeOwned + 'static,
    F,
>(
    channel: MessageType,
    f: F,
) where
    F: Fn(&MessageHeader, Option<RequestType>) -> (Option<MessageType>, Option<ResponseType>)
        + 'static,
{
    let inner_f = Rc::new(Box::new(f));

    let outer_f: Bridge = Box::new(move |header: &mut MessageHeader, raw_body: &mut [u8]| {
        let optional_body: Option<RequestType> = if header.packet_length == 0 {
            None
        } else if header.packet_length > 27 {
            debug_log(&format!(
                "[hid][{:?}] incoming packet body length is > 27 which can't be true",
                header.message_type
            ));
            return false;
        } else {
            match postcard::from_bytes::<RequestType>(raw_body) {
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
        HANDLERS.as_mut().unwrap()
    };

    // This will disable existing channels with the same message id
    // it is unlikely that there will be repeated registrations. If
    // there does become that, the handlers array can be filtered
    for (potential, handler) in handlers.iter_mut() {
        if channel.eq(potential) {
            let _ = handler.take();
        }
    }

    handlers.push((channel, Some(outer_f)));
}
