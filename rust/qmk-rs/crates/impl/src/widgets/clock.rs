use alloc::{format, string::ToString};

use crate::{
    display::Display,
    timer::Timer,
    utils::{HSV, Rect, Size},
    widgets::{UpdateOutcome, WidgetState},
};

#[derive(Debug, Default)]
pub struct State {
    pub seconds_since_midnight: u32,
    pub timer_at_set: u32,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
}

#[allow(dead_code)]
impl WidgetState<State> {
    pub fn set_seconds(&mut self, seconds: u32) -> &mut Self {
        self.state.seconds_since_midnight = seconds;
        self.state.timer_at_set = Timer::read();
        self.set_needs_redraw();
        self
    }
}

pub fn initial() -> WidgetState<State> {
    let mut state = WidgetState {
        layout_size_fn: request_size,
        update_fn: update,
        render_fn: render,
        ..Default::default()
    };
    state.set_seconds(0);
    state
}

fn request_size(display: &Display, _state: &State) -> Size {
    Size {
        width: display.bounds.size.width,
        height: display.large_font.line_height + 16,
    }
}

fn update(state: &mut State) -> UpdateOutcome {
    let actual_seconds = state.seconds_since_midnight + (Timer::elapsed(state.timer_at_set) / 1000);

    let seconds_in_minute = 60u32;
    let seconds_in_hour = 3_600u32;

    let actual_hours = ((actual_seconds / seconds_in_hour) as u8) % 24;
    let actual_seconds = actual_seconds % seconds_in_hour;
    let actual_minutes = (actual_seconds / seconds_in_minute) as u8;
    let actual_seconds = (actual_seconds % seconds_in_minute) as u8;

    let needs_display = state.hours != actual_hours || state.minutes != actual_minutes;

    state.hours = actual_hours;
    state.minutes = actual_minutes;
    state.seconds = actual_seconds;

    if needs_display {
        UpdateOutcome::RequiresRedraw
    } else {
        UpdateOutcome::NoChange
    }
}

fn render(display: &Display, state: &mut State, frame: Rect, _first_render: bool) {
    display.fill_rect(frame, *display.clear_colour);

    let text = if state.seconds_since_midnight == 0 {
        "Waiting".to_string()
    } else {
        format!("{:0>2}:{:0>2}", state.hours, state.minutes)
    };

    display.center_text(
        &display.large_font,
        frame,
        HSV::black(),
        *display.accent_colour,
        &text,
    );
}
