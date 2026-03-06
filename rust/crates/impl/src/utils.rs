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

#[derive(Copy, Clone)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    #[inline]
    pub fn at(x: u16, y: u16) -> Point {
        Point { x, y }
    }
}

#[derive(Copy, Clone)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub trait Sizeable {
    fn size(&self) -> Size;
}

pub enum Alignment {
    Leading,
    Center,
    Trailing,
}
