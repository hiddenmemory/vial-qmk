use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use once_cell::sync::Lazy;

use crate::{
    keyboard::{Channel, Keyboard, Role},
    timer::Timer,
};

const DEBUG_CHANNEL: Channel = Channel::Debug;
static mut SHOULD_ALLOW_DEBUG_OUTPUT: bool = true;
static mut SECONDARY_DEBUG_BUFFER: Lazy<Vec<String>> = Lazy::new(|| Vec::with_capacity(32));
static mut INFLIGHT_STRING: Option<String> = None;
const HEADER_SIZE: usize = 4;

pub fn debug_toggle(on: bool) {
    unsafe extern "C" {
        pub fn debug_toggle(on: bool);
    }

    if !on {
        debug_log("turning off debug output!");
    }

    unsafe {
        SHOULD_ALLOW_DEBUG_OUTPUT = on;
        debug_toggle(on);
    }

    if on {
        debug_log("debug output now on!");
    }
}

pub fn debug_log(message: &str) {
    #[allow(static_mut_refs)]
    unsafe {
        if SHOULD_ALLOW_DEBUG_OUTPUT {
            let time = Timer::read();
            let role = Keyboard::role();
            let h_start = role.highlight_start();
            let h_end = role.highlight_finish();
            let role_label = role.label();
            let side_label = Keyboard::side().label();

            let realised_message = format!(
                "{: >6}.{:0<3} {side_label}({h_start}{role_label}{h_end}) {h_start}{message}{h_end}\n\0",
                time / 1000,
                time % 1000,
            );

            match Keyboard::role() {
                Role::Primary => {
                    qmk_sys::printf(realised_message.as_ptr());
                }
                Role::Secondary => {
                    SECONDARY_DEBUG_BUFFER.push(realised_message);

                    while SECONDARY_DEBUG_BUFFER.len() > 32 {
                        let _ = SECONDARY_DEBUG_BUFFER.remove(0);
                    }
                }
            }
        }
    }
}

pub fn initialise() {
    if Keyboard::is_secondary() {
        unsafe {
            qmk_sys::transaction_register_rpc(DEBUG_CHANNEL.to_qmk_id(), Some(debug_bridge));
        }

        debug_log("debug_log initialised for secondary");
    } else {
        debug_log("debug_log initialised for primary");
    }
}

unsafe extern "C" fn debug_bridge(
    _in_len: u8,
    _in_data: *const core::ffi::c_void,
    out_len: u8,
    out_data: *mut core::ffi::c_void,
) {
    #[allow(static_mut_refs)]
    unsafe {
        let outgoing_data = alloc::slice::from_raw_parts_mut(out_data as *mut u8, out_len as usize);

        let message = if let Some(message) = INFLIGHT_STRING.take() {
            message
        } else if SECONDARY_DEBUG_BUFFER.is_empty() {
            outgoing_data.fill(0);
            return;
        } else {
            SECONDARY_DEBUG_BUFFER.remove(0)
        };

        outgoing_data[0] = 1; // This contains a message
        outgoing_data[1] = SECONDARY_DEBUG_BUFFER.len() as u8;
        let real_message = message.trim_end_matches("\n\0");
        let copy_length = real_message.len().min(out_len as usize - HEADER_SIZE);
        let remaining_bytes = real_message.len() - copy_length;
        outgoing_data[2] = copy_length as u8;
        outgoing_data[3] = remaining_bytes as u8;

        outgoing_data[HEADER_SIZE..copy_length + HEADER_SIZE]
            .copy_from_slice(&real_message.as_bytes()[0..copy_length]);

        if remaining_bytes > 0 {
            INFLIGHT_STRING = Some(real_message[copy_length..].to_string());
        }
    }
}

pub fn check_secondary_debug_queue() {
    let mut incoming_buffer = [0u8; 32];
    let mut inflight_string = Vec::with_capacity(64);

    loop {
        let result = unsafe {
            qmk_sys::transaction_rpc_exec(
                DEBUG_CHANNEL.to_qmk_id(),
                0,
                core::ptr::null(),
                incoming_buffer.len() as u8,
                incoming_buffer.as_mut_ptr() as *mut core::ffi::c_void,
            )
        };

        if result {
            let have_a_message = incoming_buffer[0] > 0;
            let have_more_messages = incoming_buffer[1] > 0;
            let string_length = incoming_buffer[2] as usize;
            let remaining = incoming_buffer[3] as usize;

            if have_a_message {
                inflight_string.extend_from_slice(
                    &incoming_buffer[HEADER_SIZE..(string_length + HEADER_SIZE)],
                );

                if remaining == 0 {
                    match str::from_utf8(&inflight_string) {
                        Ok(str) => unsafe {
                            qmk_sys::printf(format!("{str}\n\0").as_ptr());
                            inflight_string.clear();
                        },
                        Err(err) => {
                            debug_log(&format!("unable to receive message from secondary: {err}"));
                        }
                    }
                }
            }

            if have_more_messages || remaining > 0 {
                continue;
            } else {
                return;
            }
        } else {
            debug_log("[debug] unable to communicate with secondary");
            return;
        }
    }
}

pub fn time<F>(tag: &str, f: F)
where
    F: FnOnce(),
{
    let before = Timer::read();
    f();
    let after = Timer::read();
    debug_log(&format!(
        "[{tag}] took {} ms to render",
        after.saturating_sub(before)
    ));
}
