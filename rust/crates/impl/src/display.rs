use crate::{
    font::Font,
    utils::{Alignment, HSV, Point, Size, Sizeable, debug_log},
};

static mut DISPLAY: Option<Display> = None;

pub fn get() -> &'static mut Display {
    unsafe {
        #[allow(static_mut_refs)]
        DISPLAY.as_mut().unwrap()
    }
}

pub fn initialise() {
    unsafe {
        debug_log("bringing up display");

        #[allow(static_mut_refs)]
        DISPLAY.replace(Display::init());
    }
}

#[derive(Copy, Clone)]
pub struct Display {
    pub size: Size,
    pub clear_colour: HSV,
    pub device: qmk_sys::painter_device_t,
    pub small_font: Font,
    pub large_font: Font,
}

impl Display {
    pub fn init() -> Display {
        let (panel_width, panel_height) = Self::dimensions();

        let display = unsafe {
            qmk_sys::qp_st7789_make_spi_device(
                panel_width,
                panel_height,
                qmk_sys::VIK_CS,
                qmk_sys::VIK_GPIO1,
                qmk_sys::VIK_GPIO2,
                4,
                3,
            )
        };

        unsafe {
            qmk_sys::qp_init(display, qmk_sys::painter_rotation_t::QP_ROTATION_0);

            qmk_sys::qp_set_viewport_offsets(
                display,
                qmk_sys::LCD_OFFSET_X as u16,
                qmk_sys::LCD_OFFSET_Y as u16,
            );
        }

        let small_font = Font::new(unsafe { &qmk_sys::font_pixellari18 });
        let large_font = Font::new(unsafe { &qmk_sys::font_pixellari24 });

        Display {
            size: Size {
                width: panel_width,
                height: panel_height,
            },
            clear_colour: HSV::black(),
            device: display,
            small_font,
            large_font,
        }
    }

    #[inline]
    pub fn dimensions() -> (u16, u16) {
        (qmk_sys::LCD_WIDTH as u16, qmk_sys::LCD_HEIGHT as u16)
    }

    pub fn max_brightness() -> u8 {
        qmk_sys::BACKLIGHT_LEVELS as u8
    }

    pub fn set_brightness(level: u8) -> u8 {
        unsafe {
            let existing = Self::get_brightness();
            qmk_sys::backlight_level_noeeprom(level.min(Self::max_brightness()));
            existing
        }
    }

    pub fn get_brightness() -> u8 {
        unsafe { qmk_sys::get_backlight_level() }
    }

    pub fn clear(&self) {
        self.clear_to(self.clear_colour);
    }

    pub fn clear_to(&self, colour: HSV) {
        self.fill_rect(
            0,
            0,
            qmk_sys::LCD_WIDTH as u16,
            qmk_sys::LCD_HEIGHT as u16,
            colour,
        )
    }

    pub fn stroke_rect(&self, left: u16, top: u16, right: u16, bottom: u16, colour: HSV) {
        unsafe {
            _ = qmk_sys::qp_rect(
                self.device,
                left,
                top,
                right,
                bottom,
                colour.h,
                colour.s,
                colour.v,
                false,
            )
        }
    }

    pub fn fill_rect(&self, left: u16, top: u16, right: u16, bottom: u16, colour: HSV) {
        unsafe {
            _ = qmk_sys::qp_rect(
                self.device,
                left,
                top,
                right,
                bottom,
                colour.h,
                colour.s,
                colour.v,
                true,
            )
        }
    }

    pub fn position<S: Sizeable>(
        &self,
        sizeable: S,
        horizontal: Alignment,
        vertical: Alignment,
    ) -> Point {
        let size = sizeable.size();

        let x = match horizontal {
            Alignment::Leading => 0,
            Alignment::Center => self.size.width.saturating_sub(size.width) / 2,
            Alignment::Trailing => self.size.width.saturating_sub(size.width),
        };

        let y = match vertical {
            Alignment::Leading => 0,
            Alignment::Center => self.size.height.saturating_sub(size.height) / 2,
            Alignment::Trailing => self.size.height.saturating_sub(size.height),
        };

        Point::at(x, y)
    }
}
