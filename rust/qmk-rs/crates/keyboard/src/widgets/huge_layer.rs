use alloc::format;

use crate::{
    display::Display,
    images,
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
        height: 97 + (PADDING * 2),
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
    let layer = (state.active_layer.unwrap_or(0) + 1) as u16;
    let slice = Rect::new(50 * layer, 0, 50, 97);

    let location = frame.position(
        slice.size,
        crate::utils::Alignment::Center,
        crate::utils::Alignment::Center,
    );

    display.fill_rect(frame, *display.accent_colour);

    display.render_image_slice(
        location,
        &images::FONT_HUGE_A,
        slice,
        Some(*display.accent_colour),
    );
}
