use alloc::format;

use crate::{
    display::Display,
    keymap::KeyMap,
    utils::{HSV_BLACK, Rect, Size},
    widgets::{Outcome, WidgetState},
};

const PADDING: u16 = 16;

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
        height: display.huge_font.line_height + (PADDING * 2),
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

fn render(display: &Display, state: &mut State, frame: Rect, _first_render: bool) {
    let text = format!("{}", state.active_layer.unwrap_or(0) + 1);

    display.center_text(
        &display.huge_font,
        frame,
        HSV_BLACK,
        *display.accent_colour,
        &text,
    )
}
