use alloc::format;

use crate::{
    display::Display,
    fonts,
    keymap::KeyMap,
    utils::{Alignment, HSV_BLACK, HSV_WHITE, Rect, Size},
    widgets::{Outcome, WidgetState},
};

const PADDING: u16 = 4;

#[derive(Debug, Default)]
pub struct State {
    active_layer: Option<u8>,
}

pub fn initial() -> WidgetState<State> {
    WidgetState {
        layout_size_fn: request_size,
        update_fn: update,
        render_fn: render,

        ..Default::default()
    }
}

fn request_size(display: &Display, _state: &State) -> Size {
    Size {
        width: display.bounds.size.width,
        height: fonts::LARGE.height as u16 + (PADDING * 2),
    }
}

fn update(state: &mut State) -> Outcome {
    let current_layer = KeyMap::get_layer();

    if state
        .active_layer
        .map(|existing| existing != current_layer)
        .unwrap_or(true)
    {
        state.active_layer = Some(current_layer);
        Outcome::Redraw
    } else {
        Outcome::NoChange
    }
}

fn render(display: &mut Display, state: &mut State, frame: Rect, _first_render: bool) {
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

        fonts::render_aligned(
            display,
            &fonts::LARGE,
            rect,
            Alignment::Center,
            if current == layer {
                HSV_BLACK
            } else {
                HSV_WHITE
            },
            Some(if current == layer {
                *display.accent_colour
            } else {
                *display.clear_colour
            }),
            &text,
        )
    }
}
