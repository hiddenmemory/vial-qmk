use alloc::format;
use serde::{Deserialize, Serialize};

use crate::{
    display::Display,
    keyboard::{self, Channel, Keyboard},
    state::{self, Slime, State},
    utils::debug_log,
    widgets,
};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct SyncStateRequest {
    blue_index: u8,
    secondary_slime: Slime,
    seconds_since_midnight: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Default)]
struct SyncStateResponse {
    success: bool,
}

pub fn initialise() {
    if Keyboard::is_primary() {
        return;
    }

    keyboard::listen(Channel::A, |request: SyncStateRequest| {
        let state = state::get();

        state.blue_index = request.blue_index;
        state.secondary_slime = request.secondary_slime;

        let image = match state.secondary_slime {
            Slime::Green => &state.green_slime,
            Slime::Orange => &state.orange_slime,
        };

        state.widget_secondary_image.set_image(image);

        if let Some(time) = request.seconds_since_midnight {
            state.widget_clock.set_seconds(time);
        }

        SyncStateResponse { success: true }
    });
}

pub fn sync(state: &mut State) {
    if Keyboard::is_secondary() {
        return;
    }

    let result: Result<SyncStateResponse, _> = Keyboard::send(
        Channel::A,
        SyncStateRequest {
            blue_index: state.blue_index,
            secondary_slime: state.secondary_slime,
            seconds_since_midnight: if state.last_clock.has_changed() {
                state.last_clock.flush();
                Some(*state.last_clock)
            } else {
                None
            },
        },
    );

    if let Err(err) = result {
        debug_log(&format!("error: {err}"))
    }
}

pub fn update(state: &mut State) {
    state.widget_clock.update(widgets::clock::update);
}

pub fn layout(display: &Display, state: &mut State) {
    let size = widgets::clock::request_size(display, &state.widget_clock.inner);

    let Some((clock_frame, image_frame)) = display.bounds.split_v(size.height) else {
        return;
    };

    state.widget_clock.layout_frame = clock_frame;
    state.widget_secondary_image.layout_frame = image_frame;
}
pub fn render(display: &mut Display, state: &mut State) {
    state.widget_clock.render(display, widgets::clock::render);

    state
        .widget_secondary_image
        .render(display, widgets::image::render);
}
