use alloc::{format, string::ToString};

use crate::{
    display::Display,
    timer::Timer,
    utils::{HSV, Rect, Size},
    widgets::WidgetState,
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
        self.inner.seconds_since_midnight = seconds;
        self.inner.timer_at_set = Timer::read();
        self.set_needs_display();
        self
    }
}

pub fn initial() -> WidgetState<State> {
    let mut state = WidgetState {
        ..Default::default()
    };
    state.set_seconds(0);
    state
}

pub fn request_size(display: &Display, _state: &State) -> Size {
    Size {
        width: display.bounds.size.width,
        height: display.large_font.line_height + 16,
    }
}

pub fn update(state: &mut State) -> bool {
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

    needs_display
}

pub fn render(display: &Display, state: &State, frame: Rect) {
    display.fill_rect(frame, *display.clear_colour);

    let text = if state.seconds_since_midnight == 0 {
        "Waiting".to_string()
    } else {
        format!("{:0>2}:{:0>2}", state.hours, state.minutes)
    };

    super::center_text(
        display,
        &display.large_font,
        frame,
        HSV::black(),
        *display.accent_colour,
        &text,
    );
}
