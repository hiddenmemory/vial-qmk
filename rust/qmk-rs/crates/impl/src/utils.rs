use alloc::format;

pub fn debug_log(message: &str) {
    unsafe {
        qmk_sys::printf(format!("[rs] {message}\n\0").as_ptr());
    }
}

#[derive(Copy, Clone)]
pub struct HSV {
    pub h: u8,
    pub s: u8,
    pub v: u8,
}

impl HSV {
    pub fn white() -> HSV {
        HSV::from(0, 0, 100)
    }

    pub fn black() -> HSV {
        HSV::from(0, 0, 0)
    }

    pub fn papaya() -> HSV {
        HSV::from(30, 100, 100)
    }

    pub fn from(h: u16, s: u8, v: u8) -> HSV {
        let actual_h = ((h as f32 / 360.0) * 255.0) as u8;
        let actual_s = ((s as f32 / 100.0) * 255.0) as u8;
        let actual_v = ((v as f32 / 100.0) * 255.0) as u8;
        HSV {
            h: actual_h,
            s: actual_s,
            v: actual_v,
        }
    }
}

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
}

pub trait Sizeable {
    fn size(&self) -> Size;
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Sizeable for Size {
    fn size(&self) -> Size {
        *self
    }
}

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
            return None;
        }

        let top = Rect::new(self.origin.x, self.origin.y, self.size.width, y);

        let bottom = Rect::new(
            self.origin.x,
            self.origin.y + y,
            self.size.width,
            self.size.height - y,
        );

        Some((top, bottom))
    }
}
