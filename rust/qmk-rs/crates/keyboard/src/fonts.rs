use include_image::*;

use crate::{
    display::Display,
    utils::{Alignment, HSV, Point, Rect},
};

include_font!("./images/font.ttf", 140, "0123456789", LARGE_NUMBERS);
include_font!("./images/font.ttf", 24, ASCII("→"), SMALL);
include_font!("./images/font.ttf", 32, ASCII, LARGE);
pub fn width<const A: usize, const B: usize>(font: &include_image::Font<A, B>, value: &str) -> u16 {
    value.chars().fold(0, |total, c| {
        let width = if c == ' ' {
            font.space_width as u16
        } else {
            font.slice_for_char(c).map(|r| r.width as u16).unwrap_or(0)
        };
        width + font.character_padding as u16 + total
    }) - font.character_padding as u16
}

pub fn render<const A: usize, const B: usize>(
    display: &mut Display,
    font: &include_image::Font<A, B>,
    value: &str,
    at: Point,
    fg: HSV,
    bg: Option<HSV>,
) {
    let mut walking_x = 0;
    let (r, g, b) = fg.to_rgb8();
    let font_with_colour = font.with_colour(r, g, b);

    for char in value.chars() {
        let render = char != ' ';
        let Some(include_image::Rect {
            x,
            y,
            width,
            height,
        }) = font.slice_for_char(char)
        else {
            continue;
        };

        if render {
            let position = Point::at(at.x + walking_x, at.y);
            display.render_image_slice(
                position,
                &font_with_colour as &dyn include_image::Image,
                Rect::new(x as u16, y as u16, width as u16, height as u16),
                bg,
            );
        }

        walking_x += width as u16 + font.character_padding as u16;
    }
}

pub fn render_aligned<const A: usize, const B: usize>(
    display: &mut Display,
    font: &include_image::Font<A, B>,
    frame: Rect,
    alignment: Alignment,
    fg: HSV,
    bg: Option<HSV>,
    text: &str,
) {
    if let Some(bg) = bg {
        display.fill_rect(frame, bg);
    }

    let slice = Rect::new(0, 0, width(font, text), font.get_height() as u16);

    let location = frame.position(slice.size, alignment, crate::utils::Alignment::Center);

    render(display, font, text, location, fg, bg);
}
