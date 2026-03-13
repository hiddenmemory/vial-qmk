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
    fn has_alpha(&self) -> bool {
        false
    }
}

impl core::fmt::Debug for dyn Image {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Image<{},{}>", self.get_width(), self.get_height(),)
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
    pub fn with_colour(&self, r: u8, g: u8, b: u8) -> Option<ImageRecolour<'_>> {
        if self.has_alpha && self.palette_size() == 1 {
            Some(ImageRecolour::wrap(
                self.id,
                &self.pixels,
                r,
                g,
                b,
                self.width,
                self.height,
            ))
        } else {
            None
        }
    }
}

#[inline(always)]
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
    fn has_alpha(&self) -> bool {
        self.has_alpha
    }
}

pub struct ImageRecolour<'a> {
    id: u32,
    pixels: &'a [u8],
    palette: [u8; 3],
    width: usize,
    height: usize,
}

impl<'a> ImageRecolour<'a> {
    fn wrap(
        id: u32,
        image: &'a [u8],
        r: u8,
        g: u8,
        b: u8,
        width: usize,
        height: usize,
    ) -> ImageRecolour<'a> {
        ImageRecolour {
            id,
            pixels: image,
            palette: [r, g, b],
            width,
            height,
        }
    }
}

impl<'a> Image for ImageRecolour<'a> {
    fn get_id(&self) -> u32 {
        self.id
    }
    fn get_pixel(&self, x: usize, y: usize) -> Option<(u8, u8, u8, Option<u8>)> {
        if x > self.width as usize || y > self.height as usize {
            return None;
        }

        Some(get_pixel(
            x,
            y,
            self.width,
            true,
            1,
            &self.palette,
            &self.pixels,
        ))
    }
    fn get_width(&self) -> usize {
        self.width
    }
    fn get_height(&self) -> usize {
        self.height
    }
    fn has_alpha(&self) -> bool {
        true
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

pub struct Font<const CodePoints: usize, const PixelSize: usize> {
    pub id: u32,
    pub font_size: u8,
    pub space_width: u8,
    pub character_padding: u8,
    pub count: usize,
    pub width: usize,
    pub height: usize,
    pub has_alpha: bool,
    pub code_points: [(char, usize, usize); CodePoints],
    pub pixels: [u8; PixelSize],
}

pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Rect {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }
}

impl<const CodePoints: usize, const PixelSize: usize> Font<CodePoints, PixelSize> {
    pub fn slice_for_char(&self, c: char) -> Option<Rect> {
        let offset = self
            .code_points
            .binary_search_by(|(point, _, _)| point.cmp(&c))
            .ok()?;
        let (_, x, width) = self.code_points[offset];
        Some(Rect::new(x, 0, width, self.height))
    }

    pub fn with_colour(&self, r: u8, g: u8, b: u8) -> FontWithColour<'_> {
        FontWithColour {
            id: self.id,
            pixels: &self.pixels,
            palette: [r, g, b],
            width: self.width,
            height: self.width,
        }
    }
}

impl<const CodePoints: usize, const PixelSize: usize> Image for Font<CodePoints, PixelSize> {
    fn get_id(&self) -> u32 {
        self.id
    }
    fn get_pixel(&self, x: usize, y: usize) -> Option<(u8, u8, u8, Option<u8>)> {
        if x > self.width as usize || y > self.height as usize {
            return None;
        }

        let palette = [0u8, 0u8, 0u8];

        Some(get_pixel(
            x,
            y,
            self.width,
            self.has_alpha,
            1,
            &palette,
            &self.pixels,
        ))
    }
    fn get_width(&self) -> usize {
        self.width
    }
    fn get_height(&self) -> usize {
        self.height
    }
    fn has_alpha(&self) -> bool {
        self.has_alpha
    }
}

pub struct FontWithColour<'a> {
    id: u32,
    pixels: &'a [u8],
    palette: [u8; 3],
    width: usize,
    height: usize,
}

impl<'a> Image for FontWithColour<'a> {
    fn get_id(&self) -> u32 {
        self.id
    }
    fn get_pixel(&self, x: usize, y: usize) -> Option<(u8, u8, u8, Option<u8>)> {
        Some((
            self.palette[0],
            self.palette[1],
            self.palette[2],
            Some(self.pixels[(y * self.width) + x]),
        ))
    }
    fn get_width(&self) -> usize {
        self.width
    }
    fn get_height(&self) -> usize {
        self.height
    }
    fn has_alpha(&self) -> bool {
        true
    }
}
