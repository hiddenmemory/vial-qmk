use alloc::format;
use serde::{Deserialize, Serialize};

use crate::{
    display::Display,
    keyboard::{Channel, Keyboard},
    state::{self, Slime, State},
    utils::debug_log,
    widgets,
};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct SyncStateRequest {
    blue_index: u8,
    secondary_slime: Slime,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Default)]
struct SyncStateResponse {
    success: bool,
}

pub fn initialise() {
    if Keyboard::is_primary() {
        return;
    }

    Keyboard::listen(Channel::A, |request: SyncStateRequest| {
        let state = state::get();

        state.blue_index = request.blue_index;
        state.secondary_slime = request.secondary_slime;

        let image = match state.secondary_slime {
            Slime::Green => &state.green_slime,
            Slime::Orange => &state.orange_slime,
        };

        state.widget_secondary_image.set_image(image);

        SyncStateResponse { success: true }
    });
}

pub fn sync(state: &State) {
    if Keyboard::is_secondary() {
        return;
    }

    let result: Result<SyncStateResponse, _> = Keyboard::send(
        Channel::A,
        SyncStateRequest {
            blue_index: state.blue_index,
            secondary_slime: state.secondary_slime,
        },
    );

    if let Err(err) = result {
        debug_log(&format!("error: {err}"))
    }
}

pub fn update(state: &mut State) {}
pub fn layout(display: &Display, state: &mut State) {
    state.widget_secondary_image.layout_frame = display.bounds;
}
pub fn render(display: &mut Display, state: &mut State) {
    state
        .widget_secondary_image
        .render(display, widgets::image::render);
}
