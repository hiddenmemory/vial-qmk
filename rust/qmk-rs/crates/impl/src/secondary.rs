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

    state.widget_sleep_progress.set_progress_using(
        qmk_sys::QUANTUM_PAINTER_DISPLAY_TIMEOUT - Keyboard::last_activity_elapsed(),
        qmk_sys::QUANTUM_PAINTER_DISPLAY_TIMEOUT,
    );
}

pub fn layout(display: &Display, state: &mut State) {
    let clock_size = widgets::clock::request_size(display, &state.widget_clock.inner);

    let progress_size =
        widgets::progress::request_size(display, &state.widget_sleep_progress.inner);

    let Some((clock_frame, remaining_frame)) = display.bounds.split_v(clock_size.height) else {
        return;
    };

    let Some((progress_frame, image_frame)) = remaining_frame.split_v(progress_size.height) else {
        return;
    };

    state.widget_clock.layout_frame = clock_frame;
    state.widget_sleep_progress.layout_frame = progress_frame;
    state.widget_secondary_image.layout_frame = image_frame;
}
pub fn render(display: &mut Display, state: &mut State) {
    state.widget_clock.render(display, widgets::clock::render);

    state
        .widget_sleep_progress
        .render(display, widgets::progress::render);

    state
        .widget_secondary_image
        .render(display, widgets::image::render);
}
