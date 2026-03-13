use alloc::vec::Vec;
use include_image::*;

use crate::{
    display::Display,
    utils::{HSV, Point, Rect},
};

include_font!("./images/font.ttf", 140, "0123456789", LARGE_NUMBERS);
include_font!("./images/font.ttf", 24, ASCII("→"), SMALL);
include_font!("./images/font.ttf", 32, ASCII, LARGE);

pub fn slices<const A: usize, const B: usize>(
    font: &include_image::Font<A, B>,
    value: &str,
) -> Vec<(char, Rect, bool)> {
    value
        .chars()
        .flat_map(|c| {
            if c == ' ' {
                Some((c, Rect::new(0, 0, font.space_width as u16, 0), false))
            } else {
                font.slice_for_char(c).map(|r| {
                    (
                        c,
                        Rect::new(r.x as u16, r.y as u16, r.width as u16, r.height as u16),
                        true,
                    )
                })
            }
        })
        .collect()
}

pub fn width<const A: usize, const B: usize>(font: &include_image::Font<A, B>, value: &str) -> u16 {
    slices(font, value)
        .iter()
        .map(|(_, r, _)| r.size.width + font.character_padding as u16)
        .sum::<u16>()
        - font.character_padding as u16
}

pub fn render<const A: usize, const B: usize>(
    display: &Display,
    font: &include_image::Font<A, B>,
    value: &str,
    at: Point,
    fg: HSV,
    bg: Option<HSV>,
) {
    let mut walking_x = 0;
    let (r, g, b) = fg.to_rgb8();
    let font_with_colour = font.with_colour(r, g, b);

    for (_char, slice, render) in slices(font, value) {
        if render {
            let position = Point::at(at.x + walking_x, at.y);
            display.render_image_slice(
                position,
                &font_with_colour as &dyn include_image::Image,
                slice,
                bg,
            );

            // display.stroke_rect(
            //     Rect::new(position.x, position.y, slice.size.width, slice.size.height),
            //     HSV_WHITE,
            // );
        }

        walking_x += slice.size.width + font.character_padding as u16;
    }
}

pub fn render_centered<const A: usize, const B: usize>(
    display: &Display,
    font: &include_image::Font<A, B>,
    frame: Rect,
    fg: HSV,
    bg: Option<HSV>,
    text: &str,
) {
    if let Some(bg) = bg {
        display.fill_rect(frame, bg);
    }

    let slice = Rect::new(0, 0, width(font, text), font.get_height() as u16);

    let location = frame.position(
        slice.size,
        crate::utils::Alignment::Center,
        crate::utils::Alignment::Center,
    );

    render(display, font, text, location, fg, bg);
}
