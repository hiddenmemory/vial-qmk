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
    fn get_width(&self) -> usize {
        0
    }
    fn get_height(&self) -> usize {
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

pub struct ImageRGB565A<const Size: usize> {
    pub id: u32,
    pub width: usize,
    pub height: usize,
    pub has_alpha: bool,
    pub pixels: [u8; Size],
}

impl<const Size: usize> Image for ImageRGB565A<Size> {
    fn get_id(&self) -> u32 {
        self.id
    }
    fn get_pixel(&self, x: usize, y: usize) -> Option<(u8, u8, u8, Option<u8>)> {
        if x > self.width as usize || y > self.height as usize {
            return None;
        }

        let pixel_length: usize = if self.has_alpha { 3 } else { 2 };
        let base = ((y * self.width as usize) + x) * pixel_length;
        let (r, g, b) = rgb565_to_rgb888(self.pixels[base], self.pixels[base + 1]);

        let alpha = if self.has_alpha {
            Some(self.pixels[base + 2])
        } else {
            None
        };

        Some((r, g, b, alpha))
    }
    fn get_width(&self) -> usize {
        self.width
    }
    fn get_height(&self) -> usize {
        self.height
    }
    fn get_bpp(&self) -> u8 {
        16
    }
    fn has_alpha(&self) -> bool {
        self.has_alpha
    }
}

pub struct ImageRGBP256<const PixelSize: usize> {
    pub id: u32,
    pub width: usize,
    pub height: usize,
    pub has_alpha: bool,
    pub pixels: [u8; PixelSize],
}

impl<const PixelSize: usize> ImageRGBP256<PixelSize> {
    pub fn palette_size(&self) -> usize {
        let size = self.pixels[0] as usize;
        if size == 0 { 256 } else { size }
    }
    pub fn with_colour(&self, r: u8, g: u8, b: u8) -> Option<ImageRecolour<'_, PixelSize>> {
        ImageRecolour::wrap(self, r, g, b)
    }
}

fn get_pixel(
    x: usize,
    y: usize,
    width: usize,
    alpha: bool,
    palette_size: usize,
    palette: &[u8],
    pixels: &[u8],
) -> (u8, u8, u8, Option<u8>) {
    let pixel_length: usize = match (alpha, palette_size) {
        (false, 1) => 0,
        (true, 1) => 1,
        (false, _) => 1,
        (true, _) => 2,
    };

    let base = ((y * width) + x) * pixel_length;
    let index = if palette_size == 1 {
        0
    } else {
        pixels[base] as usize
    };

    let palette_base = index * 3;
    let r = palette[palette_base];
    let g = palette[palette_base + 1];
    let b = palette[palette_base + 2];

    let alpha = if alpha && palette_size > 1 {
        Some(pixels[base + 1])
    } else if alpha && palette_size == 1 {
        Some(pixels[base])
    } else {
        None
    };

    (r, g, b, alpha)
}

impl<const PixelSize: usize> Image for ImageRGBP256<PixelSize> {
    fn get_id(&self) -> u32 {
        self.id
    }
    fn get_pixel(&self, x: usize, y: usize) -> Option<(u8, u8, u8, Option<u8>)> {
        if x > self.width as usize || y > self.height as usize {
            return None;
        }

        let palette_length: usize = self.palette_size();
        let (_, palette) = self.pixels.split_at(1);
        let (palette, pixels) = palette.split_at(palette_length * 3);

        Some(get_pixel(
            x,
            y,
            self.width,
            self.has_alpha,
            self.palette_size(),
            palette,
            pixels,
        ))
    }
    fn get_width(&self) -> usize {
        self.width
    }
    fn get_height(&self) -> usize {
        self.height
    }
    fn get_bpp(&self) -> u8 {
        32
    }
    fn has_alpha(&self) -> bool {
        self.has_alpha
    }
}

pub struct ImageRecolour<'a, const Size: usize> {
    source: &'a ImageRGBP256<Size>,
    palette: [u8; 3],
}

impl<'a, const Size: usize> ImageRecolour<'a, Size> {
    pub fn wrap(
        image: &'a ImageRGBP256<Size>,
        r: u8,
        g: u8,
        b: u8,
    ) -> Option<ImageRecolour<'a, Size>> {
        if image.palette_size() != 1 {
            None
        } else {
            Some(ImageRecolour {
                source: image,
                palette: [r, g, b],
            })
        }
    }
}

impl<'a, const PixelSize: usize> Image for ImageRecolour<'a, PixelSize> {
    fn get_id(&self) -> u32 {
        self.source.id
    }
    fn get_pixel(&self, x: usize, y: usize) -> Option<(u8, u8, u8, Option<u8>)> {
        if x > self.source.width as usize || y > self.source.height as usize {
            return None;
        }

        let palette_length: usize = self.source.palette_size();
        let (_, palette) = self.source.pixels.split_at(1);
        let (_, pixels) = palette.split_at(palette_length * 3);

        Some(get_pixel(
            x,
            y,
            self.source.width,
            self.source.has_alpha,
            1,
            &self.palette,
            pixels,
        ))
    }
    fn get_width(&self) -> usize {
        self.source.width
    }
    fn get_height(&self) -> usize {
        self.source.height
    }
    fn get_bpp(&self) -> u8 {
        self.source.get_bpp()
    }
    fn has_alpha(&self) -> bool {
        self.source.has_alpha
    }
}

#[inline(always)]
pub fn blend_pixel(fg: u8, bg: u8, alpha: Option<u8>) -> u8 {
    // This was lifted from: gdk-pixbuf:
    // https://gitlab.gnome.org/GNOME/gdk-pixbuf/-/blob/5a5d37bd6696c96d5567c2199cac0fbc5b86d0e8/gdk-pixbuf/pixops/pixops.c#L407-411
    let r_src: u16 = fg as u16;
    let r_dst: u16 = bg as u16;
    let a0 = alpha.unwrap_or(0xFF) as u16;
    let a1 = 0xff - a0;
    let tmp = a0 * r_src + a1 * r_dst + 0x80;
    ((tmp + (tmp >> 8)) >> 8) as u8
}

#[inline(always)]
pub fn rgb888_to_rgb565(red: u8, green: u8, blue: u8) -> (u8, u8) {
    (
        (red & 0xF8) | (green >> 5),
        (green & 0b11111100) << 3 | (blue >> 3),
    )
}

#[inline(always)]
pub fn rgb565_to_rgb888(h_byte: u8, l_byte: u8) -> (u8, u8, u8) {
    (
        h_byte & 0xF8,
        (h_byte << 5) | (l_byte >> 5 << 2),
        l_byte << 3,
    )
}
