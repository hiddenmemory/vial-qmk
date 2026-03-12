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
