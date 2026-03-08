use crate::state::Slime;
use alloc::format;
use hid_bridge::{Empty, HsvValue, MessageType};

use crate::keymap::KeyMap;
use crate::{
    display,
    keyboard::{Keyboard, Role},
    state, usb,
    utils::debug_log,
};

pub fn initialise() {
    if !Keyboard::is_primary() {
        return;
    }

    usb::listen::<Empty, HsvValue, _>(MessageType::QueryRgbHsv, |_, _| {
        let hsv = unsafe { qmk_sys::rgb_matrix_get_hsv() };

        debug_log(&format!(
            "RGB HSV current values: (h: {}, s: {}, v: {})",
            hsv.h, hsv.s, hsv.v
        ));

        (
            Some(MessageType::QueryRgbHsv),
            Some(HsvValue {
                h: hsv.v as u16,
                s: hsv.s,
                v: hsv.v,
            }),
        )
    });

    usb::listen::<HsvValue, Empty, _>(MessageType::SetRgbHsv, |_, values| {
        if let Some(hsv) = values {
            debug_log(&format!(
                "Setting RGB HSV current values: (h: {}, s: {}, v: {})",
                hsv.h, hsv.s, hsv.v
            ));

            unsafe {
                qmk_sys::rgb_matrix_sethsv(hsv.h, hsv.s, hsv.v);
            }
        }

        (Some(MessageType::Acknowledge), None)
    });

    unsafe {
        qmk_sys::rgb_matrix_mode_noeeprom(0);
    }
}

#[inline]
fn scale_value(value: u8, by: u8) -> u8 {
    let max_value = qmk_sys::RGB_MATRIX_MAXIMUM_BRIGHTNESS;
    ((value as f32) * (by as f32 / max_value as f32)) as u8
}

#[unsafe(no_mangle)]
pub extern "C" fn rgb_matrix_indicators_advanced_rs(min: u8, max: u8) {
    unsafe {
        let hsv_v = qmk_sys::rgb_matrix_get_val();
        let state = state::get();
        let display_off = display::get().power_level.get().is_off();
        let value = scale_value(0xFF, hsv_v);

        let (red, green, blue): (u8, u8, u8) = if display_off {
            (0x01, 0x01, 0x01)
        } else if KeyMap::get_layer() == 0
            && (matches!(Keyboard::role(), Role::Primary)
                || matches!(state.secondary_slime.get(), Slime::Green))
        {
            (0x0, value, 0x0)
        } else {
            (value, value / 2, 0x0)
        };

        let blue_index = state.blue_index.get();

        for offset in min..=max {
            let (red, green, blue) = if offset == blue_index {
                (0x0, 0x0, value)
            } else {
                (red, green, blue)
            };

            qmk_sys::rgb_matrix_set_color(offset as i32, red, green, blue);
        }
    }
}
