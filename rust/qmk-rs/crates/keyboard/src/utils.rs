pub mod debug;

#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct HSV {
    pub h: u8,
    pub s: u8,
    pub v: u8,
}

impl HSV {
    pub const fn from(h: u16, s: u8, v: u8) -> HSV {
        let actual_h = ((h as f32 / 360.0) * 255.0) as u8;
        let actual_s = ((s as f32 / 100.0) * 255.0) as u8;
        let actual_v = ((v as f32 / 100.0) * 255.0) as u8;
        HSV {
            h: actual_h,
            s: actual_s,
            v: actual_v,
        }
    }

    #[allow(dead_code)]
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> HSV {
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        // Value
        let v = (max / 255.0 * 100.0) as u8;

        // Saturation
        let s = if v == 0 {
            0
        } else {
            (delta / max * 100.0) as u8
        };

        // Hue
        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        // Normalize hue to [0, 360)
        let h = if h < 0.0 { h + 360.0 } else { h };

        HSV::from(h as u16, s, v)
    }

    pub fn to_rgb8(self) -> (u8, u8, u8) {
        if self.s == 0 {
            return (self.v, self.v, self.v);
        }

        let h = (self.h as f32 / 255.0) * 360.0;
        let s = self.s as f32 / 255.0;
        let v = self.v as f32 / 255.0;

        let h = h / 60.0;
        let i = h as u32;
        let f = h - (h as u32) as f32;

        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));

        let (r, g, b) = match i % 6 {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            5 => (v, p, q),
            _ => unreachable!(),
        };

        ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }
}

pub static HSV_WHITE: HSV = HSV::from(0, 0, 100);
pub static HSV_BLACK: HSV = HSV::from(0, 0, 0);
pub static HSV_ORANGE: HSV = HSV::from(30, 100, 100);
pub static HSV_LIME: HSV = HSV::from(105, 89, 95);

#[derive(Default, Debug, Copy, Clone)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub fn zero() -> Point {
        Point::at(0, 0)
    }
    #[inline]
    pub fn at(x: u16, y: u16) -> Point {
        Point { x, y }
    }

    pub fn shift_v(&mut self, amount: i16) {
        if amount < 0 {
            self.y = self.y.saturating_sub(amount.unsigned_abs());
        } else {
            self.y = self.y.saturating_add(amount as u16);
        }
    }
}

pub trait Sizeable {
    fn size(&self) -> Size;
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Sizeable for Size {
    fn size(&self) -> Size {
        *self
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Default, Debug)]
pub enum Alignment {
    Leading,
    #[default]
    Center,
    Trailing,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Rect {
        Rect {
            origin: Point { x, y },
            size: Size { width, height },
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

        Point::at(x + self.origin.x, y + self.origin.y)
    }

    pub fn as_qp(&self) -> (u16, u16, u16, u16) {
        (
            self.origin.x,
            self.origin.y,
            self.origin.x + self.size.width,
            self.origin.y + self.size.height,
        )
    }

    pub fn split_v(&self, y: u16) -> Option<(Rect, Rect)> {
        // We can't split if the split point isn't within this rect
        if y > self.size.height {
            return Some((
                *self,
                Rect::new(
                    self.origin.x,
                    self.origin.y + self.size.height + 1,
                    self.size.width,
                    0,
                ),
            ));
        }

        let top = Rect::new(self.origin.x, self.origin.y, self.size.width, y);

        let bottom = Rect::new(
            self.origin.x,
            self.origin.y + y + 1,
            self.size.width,
            self.size.height - y,
        );

        Some((top, bottom))
    }
}

pub struct TrackValue<Value: Eq> {
    value: Value,
    has_changed: bool,
}

impl<Value: Eq> TrackValue<Value> {
    pub fn new(value: Value) -> TrackValue<Value> {
        TrackValue {
            value,
            has_changed: true,
        }
    }

    pub fn set(&mut self, value: Value) {
        if self.value.eq(&value) {
            return;
        }

        self.value = value;
        self.has_changed = true;
    }

    #[inline]
    pub fn has_changed(&self) -> bool {
        self.has_changed
    }

    pub fn flush(&mut self) {
        self.has_changed = false;
    }
}

impl<Value: Eq> core::ops::Deref for TrackValue<Value> {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<Value: Eq> core::ops::DerefMut for TrackValue<Value> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub fn calculate_time(seconds: u32) -> (u8, u8, u8) {
    let seconds_in_minute = 60u32;
    let seconds_in_hour = 3_600u32;

    let hours = ((seconds / seconds_in_hour) as u8) % 24;
    let actual_seconds = seconds % seconds_in_hour;
    let minutes = (actual_seconds / seconds_in_minute) as u8;
    let seconds = (actual_seconds % seconds_in_minute) as u8;

    (hours, minutes, seconds)
}
