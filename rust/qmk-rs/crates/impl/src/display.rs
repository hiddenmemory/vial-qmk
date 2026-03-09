use alloc::vec::Vec;
use alloc::{ffi::CString, vec};

use crate::utils::{HSV_BLACK, HSV_ORANGE};
use crate::{
    font::Font,
    keyboard::Keyboard,
    sync::{SyncKey, SyncValue},
    utils::{HSV, Point, Rect, Size, TrackValue, debug_log},
};

static mut DISPLAY: Option<Display> = None;

pub fn get() -> &'static mut Display {
    unsafe {
        #[allow(static_mut_refs)]
        DISPLAY.as_mut().unwrap()
    }
}

pub fn initialise() -> &'static mut Display {
    unsafe {
        debug_log("bringing up display");

        #[allow(static_mut_refs)]
        DISPLAY.replace(Display::new());
    }

    get()
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PowerLevel {
    On(u8),
    Off(u8),
}

impl PowerLevel {
    pub fn is_off(&self) -> bool {
        matches!(self, PowerLevel::Off(_))
    }

    pub fn is_on(&self) -> bool {
        matches!(self, PowerLevel::On(_))
    }

    pub fn level(&self) -> u8 {
        match self {
            PowerLevel::On(level) => *level,
            PowerLevel::Off(level) => *level,
        }
    }
}

impl Default for PowerLevel {
    fn default() -> Self {
        Self::Off(0)
    }
}

pub struct Display {
    pub bounds: Rect,
    pub clear_colour: TrackValue<HSV>,
    pub accent_colour: TrackValue<HSV>,
    pub device: qmk_sys::painter_device_t,
    pub power_level: SyncValue<PowerLevel>,
    #[allow(dead_code)]
    device_buffer: Vec<u8>,
    actual_device: qmk_sys::painter_device_t,
    pub small_font: Font,
    pub large_font: Font,
}

#[allow(dead_code)]
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
            clear_colour: TrackValue::new(HSV_BLACK),
            accent_colour: TrackValue::new(HSV_ORANGE),
            device,
            power_level: SyncValue::new(SyncKey::DisplayPowerLevel, Default::default()),
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

    pub fn assume_off(&mut self) {
        let off_value = match self.power_level.get() {
            PowerLevel::On(level) => PowerLevel::Off(level),
            level => level,
        };

        self.power_level.set(off_value);

        unsafe {
            qmk_sys::backlight_level_noeeprom(0);
        }
    }

    pub fn assume_on(&mut self) {
        let on_value = match self.power_level.get() {
            PowerLevel::Off(level) => PowerLevel::On(level),
            level => level,
        };

        self.power_level.set(on_value);

        unsafe {
            qmk_sys::backlight_level_noeeprom(on_value.level());
        }
    }

    pub fn set_brightness(&mut self, level: u8) -> u8 {
        if self.power_level.get().is_on() && self.get_brightness() == level {
            return level;
        }

        unsafe {
            if self.power_level.get().is_off() && level > 0 {
                Keyboard::trigger_fake_activity();
            }

            let actual_level = level.min(Self::max_brightness());
            self.power_level.set(PowerLevel::On(actual_level));

            qmk_sys::backlight_level_noeeprom(actual_level);
            actual_level
        }
    }

    pub fn get_brightness(&self) -> u8 {
        self.power_level.get().level()
    }

    pub fn read_brightness(&self) -> u8 {
        unsafe { qmk_sys::get_backlight_level() }
    }

    pub fn requires_repaint(&self) -> bool {
        self.clear_colour.has_changed() || self.accent_colour.has_changed()
    }

    pub fn clear(&self) {
        self.fill(*self.clear_colour);
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

    pub fn center_text(&self, font: &Font, frame: Rect, fg: HSV, bg: HSV, text: &str) {
        unsafe {
            let Ok(actual_text) = CString::new(text) else {
                return;
            };

            self.fill_rect(frame, bg);

            let position = frame.position(
                font.size_of(&actual_text),
                crate::utils::Alignment::Center,
                crate::utils::Alignment::Center,
            );

            qmk_sys::qp_drawtext_recolor(
                self.device,
                position.x,
                position.y + 2,
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

    pub fn flush(&mut self) {
        self.clear_colour.flush();
        self.accent_colour.flush();

        unsafe {
            qmk_sys::qp_surface_draw(self.device, self.actual_device, 0, 0, false);
        }
    }
}
