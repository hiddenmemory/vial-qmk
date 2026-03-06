use alloc::format;

use crate::{display::Display, keymap::KeyMap, state::State, utils::HSV};

pub fn render(display: &Display, _state: &mut State) {
    let lcd_width = display.size.width;
    let top: u16 = 10 + display.small_font.line_height + 4;

    let layer_width = (lcd_width as f32 / KeyMap::layer_count() as f32) as u16;
    let left_padding = (lcd_width - (layer_width * KeyMap::layer_count() as u16)) / 2;
    let current = KeyMap::get_layer();

    for layer in 0..KeyMap::layer_count() {
        let label = format!("{}", layer + 1);

        super::center_text(
            display,
            &display.large_font,
            left_padding + (layer_width * layer as u16),
            top,
            layer_width,
            if current == layer {
                HSV::black()
            } else {
                HSV::white()
            },
            if current == layer {
                HSV::papaya()
            } else {
                HSV::black()
            },
            &label,
        )
    }
}
