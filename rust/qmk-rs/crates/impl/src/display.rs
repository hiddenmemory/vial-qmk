use alloc::vec;
use alloc::vec::Vec;

use crate::{
    font::Font,
    utils::{HSV, Point, Rect, Size, debug_log},
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
        DISPLAY.replace(Display::new());
    }
}

pub struct Display {
    pub bounds: Rect,
    pub clear_colour: HSV,
    pub device: qmk_sys::painter_device_t,
    device_buffer: Vec<u8>,
    actual_device: qmk_sys::painter_device_t,
    pub small_font: Font,
    pub large_font: Font,
}

impl Display {
    pub fn new() -> Display {
        let (panel_width, panel_height) = Self::dimensions();

        let actual_device = unsafe {
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
            qmk_sys::qp_init(actual_device, qmk_sys::painter_rotation_t::QP_ROTATION_0);

            qmk_sys::qp_set_viewport_offsets(
                actual_device,
                qmk_sys::LCD_OFFSET_X as u16,
                qmk_sys::LCD_OFFSET_Y as u16,
            );
        }

        let small_font = Font::new(unsafe { &qmk_sys::font_pixellari18 });
        let large_font = Font::new(unsafe { &qmk_sys::font_pixellari24 });

        let mut device_buffer = vec![0_u8; 64_801];
        let device = unsafe {
            qmk_sys::qp_make_rgb565_surface(
                panel_width,
                panel_height,
                device_buffer.as_mut_ptr() as *mut core::ffi::c_void,
            )
        };

        unsafe {
            qmk_sys::qp_init(device, qmk_sys::painter_rotation_t::QP_ROTATION_0);
        }

        Display {
            bounds: Rect {
                origin: Point::zero(),
                size: Size {
                    width: panel_width,
                    height: panel_height,
                },
            },
            clear_colour: HSV::black(),
            device,
            device_buffer,
            actual_device,
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
        self.fill(self.clear_colour);
    }

    pub fn fill(&self, colour: HSV) {
        self.fill_rect(self.bounds, colour)
    }

    pub fn stroke_rect(&self, rect: Rect, colour: HSV) {
        unsafe {
            let (left, top, right, bottom) = rect.as_qp();

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

    pub fn fill_rect(&self, rect: Rect, colour: HSV) {
        unsafe {
            let (left, top, right, bottom) = rect.as_qp();

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

    pub fn flush(&self) {
        unsafe {
            qmk_sys::qp_surface_draw(self.device, self.actual_device, 0, 0, false);
        }
    }
}
