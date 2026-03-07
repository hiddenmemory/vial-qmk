use alloc::ffi::CString;

use crate::{
    display::Display,
    font::Font,
    utils::{HSV, Rect},
};

pub mod image;
pub mod layer;
pub mod os;

#[derive(Debug, Default)]
pub struct WidgetState<Inner: Default + core::fmt::Debug> {
    pub inner: Inner,
    pub requires_repaint: bool,
    pub layout_frame: Rect,
}

impl<Inner: Default + core::fmt::Debug> WidgetState<Inner> {
    pub fn set_needs_display(&mut self) {
        self.requires_repaint = true;
    }

    pub fn update<F>(&mut self, f: F)
    where
        F: Fn(&mut Inner) -> bool,
    {
        if f(&mut self.inner) {
            self.set_needs_display();
        }
    }

    pub fn render<F>(&mut self, display: &Display, f: F)
    where
        F: Fn(&Display, &Inner, Rect),
    {
        if self.requires_repaint {
            f(display, &self.inner, self.layout_frame);
            self.requires_repaint = false;
        }
    }
}

pub fn center_text(display: &Display, font: &Font, frame: Rect, fg: HSV, bg: HSV, text: &str) {
    unsafe {
        let Ok(actual_text) = CString::new(text) else {
            return;
        };

        display.fill_rect(frame, bg);

        let position = frame.position(
            font.size_of(&actual_text),
            crate::utils::Alignment::Center,
            crate::utils::Alignment::Center,
        );

        qmk_sys::qp_drawtext_recolor(
            display.device,
            position.x,
            position.y,
            font.handle,
            actual_text.as_ptr(),
            fg.h,
            fg.s,
            fg.v,
            bg.h,
            bg.s,
            bg.v,
        );
    }
}
