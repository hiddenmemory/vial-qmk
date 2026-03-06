use alloc::ffi::CString;

use crate::{display::Display, font::Font, utils::HSV};

pub mod layer;
pub mod os;

pub fn center_text(
    display: &Display,
    font: &Font,
    x: u16,
    y: u16,
    width: u16,
    fg: HSV,
    bg: HSV,
    text: &str,
) {
    unsafe {
        let Ok(actual_text) = CString::new(text) else {
            return;
        };

        let actual_pointer = actual_text.as_ptr();
        let text_width = qmk_sys::qp_textwidth(font.handle, actual_pointer);

        qmk_sys::qp_drawtext_recolor(
            display.device,
            x.saturating_add(width.saturating_sub_signed(text_width) / 2),
            y,
            font.handle,
            actual_pointer,
            fg.h,
            fg.s,
            fg.v,
            bg.h,
            bg.s,
            bg.v,
        );
    }
}
