use alloc::format;
use hid_bridge::{BoolValue, DateTime, Empty, MessageType, U32Value};

use crate::{
    eeprom::EEPROM, heap, keyboard::Keyboard, state, sync::syncing::impl_serde::MakeSyncableValue,
    utils::debug::debug_log,
};

impl MakeSyncableValue for DateTime {}

pub(crate) fn listen_for_heap_usage() {
    super::listen_and_forward::<Empty, Empty, _>(MessageType::HeapUsage, true, |_, _| {
        let (total, used, free) = heap::usage();

        debug_log(&format!(
            "request for memory usage: total={total}, used={used}, free={free} ({}%)",
            heap::usage_percentage()
        ));

        (Some(MessageType::Acknowledge), None)
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
        state::get()
            .debug_output
            .set(value.map(|value| value.value).unwrap_or(true));
        (Some(MessageType::Acknowledge), None)
    });
}

pub(crate) fn listen_for_ping() {
    super::listen::<Empty, Empty, _>(MessageType::Ping, |_, _| {
        (Some(MessageType::Acknowledge), None)
    });
}

pub(crate) fn listen_for_wake() {
    super::listen::<hid_bridge::Empty, hid_bridge::Empty, _>(MessageType::WakeDisplays, |_, _| {
        crate::check_display_state_and_render();
        Keyboard::trigger_fake_activity();
        (Some(MessageType::Acknowledge), None)
    });
}

pub(crate) fn listen_for_display_brightness() {
    super::listen_and_forward::<hid_bridge::U8ValueWithFlag, hid_bridge::Empty, _>(
        MessageType::SetDisplayBrightness,
        true,
        |_, value| {
            if let Some(value) = value {
                if Keyboard::is_primary() {
                    state::get()
                        .display_brightness
                        .set(value.value.min(qmk_sys::BACKLIGHT_LEVELS as u8));
                }

                if value.flag {
                    unsafe {
                        qmk_sys::backlight_level(value.value);
                    }

                    EEPROM::set_backlight(value.value);
                }
            }
            (Some(MessageType::Acknowledge), None)
        },
    );
}

pub(crate) fn listen_for_date_time() {
    super::listen::<hid_bridge::DateTime, hid_bridge::Empty, _>(
        MessageType::DateTime,
        |_, request: Option<hid_bridge::DateTime>| {
            let state = state::get();

            if let Some(value) = request {
                fn output(tag: &str, seconds: u32) {
                    let (h, m, s) = crate::utils::calculate_time(seconds);
                    debug_log(&format!("{tag} = {h:0>2}:{m:0>2}:{s:0>2}"));
                }

                output("since midnight", value.seconds_since_midnight);
                output("dawn", value.dawn_secs);
                output("sunrise", value.sunrise_secs);
                output("sunset", value.sunset_secs);
                output("dusk", value.dusk_secs);

                state.page_clock.set_seconds(value.seconds_since_midnight);
                state.date_time.set(value);
            }

            (Some(MessageType::Acknowledge), None)
        },
    );
}
