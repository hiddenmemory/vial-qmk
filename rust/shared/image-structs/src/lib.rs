#![no_std]
#![no_builtins]
#![crate_type = "staticlib"]
#![allow(warnings)]

pub trait Image {
    fn get_id(&self) -> u32 {
        0
    }
    fn get_pixel(&self, x: usize, y: usize) -> Option<(u8, u8, u8, Option<u8>)> {
        None
    }
    fn get_width(&self) -> u8 {
        0
    }
    fn get_height(&self) -> u8 {
        0
    }
    fn get_bpp(&self) -> u8 {
        0
    }
    fn has_alpha(&self) -> bool {
        false
    }
}

impl core::fmt::Debug for dyn Image {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Image<{},{},{}>",
            self.get_width(),
            self.get_height(),
            self.get_bpp()
        )
    }
}

pub struct ImageBRG565A<const Size: usize> {
    pub id: u32,
    pub width: u8,
    pub height: u8,
    pub has_alpha: bool,
    pub pixels: [u8; Size],
}

impl<const Size: usize> Image for ImageBRG565A<Size> {
    fn get_id(&self) -> u32 {
        self.id
    }
    fn get_pixel(&self, x: usize, y: usize) -> Option<(u8, u8, u8, Option<u8>)> {
        if x > self.width as usize || y > self.height as usize {
            return None;
        }

        let pixel_length: usize = if self.has_alpha { 3 } else { 2 };

        let base = ((y * self.width as usize) + x) * pixel_length;

        let h_byte = self.pixels[base];
        let l_byte = self.pixels[base + 1];

        let alpha = if self.has_alpha {
            Some(self.pixels[base + 2])
        } else {
            None
        };

        Some((
            h_byte & 0xF8,
            (h_byte << 5) | (l_byte >> 5 << 2),
            l_byte << 3,
            alpha,
        ))
    }
    fn get_width(&self) -> u8 {
        self.width
    }
    fn get_height(&self) -> u8 {
        self.height
    }
    fn get_bpp(&self) -> u8 {
        16
    }
    fn has_alpha(&self) -> bool {
        self.has_alpha
    }
}
