use alloc::format;
use hid_bridge::{BoolValue, Empty, MessageType, U32Value};

use crate::{heap, state, utils::debug_log};

pub(crate) fn listen_for_heap_usage() {
    super::listen::<Empty, Empty, _>(MessageType::HeapUsage, |_, _| {
        let (total, used, free) = heap::usage();

        debug_log(&format!(
            "request for memory usage: total={total}, used={used}, free={free} ({}%)",
            heap::usage_percentage()
        ));

        (None, None)
    });
}

pub(crate) fn listen_for_set_frame_time() {
    super::listen::<U32Value, Empty, _>(MessageType::SetFrameTime, |_, value| {
        let frame_time = value.map(|inner| inner.value).unwrap_or(0);
        let state = state::get();
        let current_frame_time = state.frame_time.get();

        if frame_time == 0 {
            debug_log(&format!("[usb] current frame time is {current_frame_time}"));
        } else {
            debug_log(&format!(
                "[usb] setting frame time to {frame_time} from {current_frame_time}"
            ));
            state.frame_time.set(frame_time);
        }

        (Some(MessageType::Acknowledge), None)
    });
}

pub(crate) fn listen_for_toggle_debug() {
    super::listen::<BoolValue, Empty, _>(MessageType::ToggleDebug, |_, value| {
        crate::utils::debug_toggle(value.map(|inner| inner.value).unwrap_or(true));
        (Some(MessageType::Acknowledge), None)
    });
}

pub(crate) fn listen_for_ping() {
    super::listen::<Empty, Empty, _>(MessageType::Ping, |_, _| {
        (Some(MessageType::Acknowledge), None)
    });
}
