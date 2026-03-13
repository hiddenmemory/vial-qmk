use alloc::vec::Vec;
use alloc::{ffi::CString, vec};

use crate::state;
use crate::utils::debug::debug_log;
use crate::utils::{HSV_BLACK, HSV_ORANGE};
use crate::{
    font::QmkFont,
    keyboard::Keyboard,
    sync::{SyncKey, SyncValue},
    utils::{HSV, Point, Rect, Size, TrackValue},
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
            power_level: SyncValue::with_fn(
                SyncKey::DisplayPowerLevel,
                Default::default(),
                |previous, current| {
                    if previous.is_off()
                        && current.is_on()
                        && let Some(state) = state::try_get()
                    {
                        debug_log("display coming back on, forcing a redraw");
                        state.requires_redraw()
                    }
                },
            ),
            device_buffer,
            actual_device,
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

    pub fn center_text(&self, font: &QmkFont, frame: Rect, fg: HSV, bg: HSV, text: &str) {
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
            qmk_sys::qp_flush(self.actual_device);
            qmk_sys::qp_flush(self.device);
        }
    }

    pub fn reset_clip(&self) {
        self.set_clip(self.bounds);
    }

    pub fn set_clip(&self, rect: Rect) {
        unsafe {
            qmk_sys::qp_viewport(
                self.device,
                rect.origin.x,
                rect.origin.y,
                rect.origin.x + rect.size.width - 1,
                rect.origin.y + rect.size.height - 1,
            );
        }
    }

    pub fn render_image(&self, position: Point, image: &dyn include_image::Image, bg: Option<HSV>) {
        self.render_image_slice(
            position,
            image,
            Rect::new(0, 0, image.get_width() as u16, image.get_height() as u16),
            bg,
        );
    }

    pub fn render_image_slice(
        &self,
        position: Point,
        image: &dyn include_image::Image,
        slice: Rect,
        bg: Option<HSV>,
    ) {
        if position.x >= self.bounds.size.width || position.y >= self.bounds.size.height {
            return;
        }

        let width = slice.size.width;
        let height = slice.size.height;

        let render_bottom = position.y + height;
        let render_right = position.x + width;

        let height = if render_bottom > self.bounds.size.height {
            height - (render_bottom - self.bounds.size.height)
        } else {
            height
        };

        let width = if render_right > self.bounds.size.width {
            width - (render_right - self.bounds.size.width)
        } else {
            width
        };

        // Allocate a buffer to flip onto the surface
        let mut buf = vec![0u8; width as usize * height as usize * 2];

        // Pre-calculate a background colour as to_rgb needs only doing once if we have a colour
        let (maybe_bgr, maybe_bgg, maybe_bgb) = if let Some(bg) = bg {
            bg.to_rgb8()
        } else {
            (0, 0, 0)
        };

        for y in slice.origin.y..(slice.origin.y + height) {
            for x in slice.origin.x..(slice.origin.x + width) {
                let Some((fg_r, fg_g, fg_b, fg_a)) = image.get_pixel(x as usize, y as usize) else {
                    continue;
                };

                // Get the background image for the pixel
                let (bg_r, bg_g, bg_b) = if bg.is_some() {
                    // Use pre-calculated if we have been given an image
                    (maybe_bgr, maybe_bgg, maybe_bgb)
                } else {
                    // Work out where in our self.device_buffer the backing pixel is
                    let backing_x = position.x as usize + (x - slice.origin.x) as usize;
                    let backing_y = position.y as usize + (y - slice.origin.y) as usize;
                    let backing_offset =
                        ((backing_y * self.bounds.size.width as usize) + backing_x) * 2;

                    include_image::rgb565_to_rgb888(
                        self.device_buffer[backing_offset],
                        self.device_buffer[backing_offset + 1],
                    )
                };

                // Blend the pixels...
                let red = include_image::blend_pixel(fg_r, bg_r, fg_a);
                let green = include_image::blend_pixel(fg_g, bg_g, fg_a);
                let blue = include_image::blend_pixel(fg_b, bg_b, fg_a);

                // ... get the offset ...
                let offset = (((y - slice.origin.y) as usize * width as usize)
                    + (x - slice.origin.x) as usize)
                    * 2;

                let (high, low) = include_image::rgb888_to_rgb565(red, green, blue);

                // Update the buffer!
                buf[offset] = high;
                buf[offset + 1] = low;
            }
        }

        // Set the clip rect
        self.set_clip(Rect::new(position.x, position.y, width, height));

        // Push the buffer to the display
        unsafe {
            qmk_sys::qp_pixdata(
                self.device,
                buf.as_ptr() as *const core::ffi::c_void,
                width as u32 * height as u32,
            );
        }

        self.reset_clip();
    }
}
