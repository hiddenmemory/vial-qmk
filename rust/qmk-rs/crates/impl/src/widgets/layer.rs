use alloc::format;

use crate::{
    display::Display,
    keymap::KeyMap,
    utils::{HSV, Rect, Size},
};

const PADDING: u16 = 6;

#[derive(Debug, Default)]
pub struct State {
    active_layer: Option<u8>,
}

pub fn request_size(display: &Display, _state: &State) -> Size {
    Size {
        width: display.bounds.size.width,
        height: display.small_font.line_height + (PADDING * 2),
    }
}

pub fn update(state: &mut State) -> bool {
    let current_layer = KeyMap::get_layer();

    if state
        .active_layer
        .map(|existing| existing != current_layer)
        .unwrap_or(true)
    {
        state.active_layer = Some(current_layer);
        true
    } else {
        false
    }
}

pub fn render(display: &Display, state: &State, frame: Rect) {
    let layer_count = KeyMap::layer_count() as u16;
    let layer_width = frame.size.width / layer_count;
    let padding = frame.size.width % layer_count / 2;
    let current = state.active_layer.unwrap_or(KeyMap::get_layer()) as u16;

    for layer in 0..layer_count {
        let text = format!("{}", layer + 1);

        let rect = Rect::new(
            frame.origin.x + padding + (layer * layer_width),
            frame.origin.y,
            layer_width,
            frame.size.height,
        );

        super::center_text(
            display,
            &display.large_font,
            rect,
            if current == layer {
                HSV::black()
            } else {
                HSV::white()
            },
            if current == layer {
                *display.accent_colour
            } else {
                HSV::black()
            },
            &text,
        )
    }
}
