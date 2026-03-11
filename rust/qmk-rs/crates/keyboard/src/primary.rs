use crate::{
    display::{self},
    state::State,
    timer::Timer,
};

pub fn initialise() {}

pub fn check_screen_fades(state: &mut State) {
    let display = display::get();

    // 0 means we have hit the timeout
    let idle_remaining = Timer::idle_time_remaining();

    let duration = state.display_brightness.get() as u32 * 100;

    // If we are half a second to switching off the display
    if state.screen_fade_out.finished() && idle_remaining > 0 && idle_remaining < duration {
        state.screen_fade_out.duration = idle_remaining;
        state.screen_fade_out.restart();
    }

    if state.screen_fade_out.running() {
        display.set_brightness(display.get_brightness().min(state.screen_fade_out.next()));
    }

    if state.screen_fade_in.running() {
        display.set_brightness(display.get_brightness().max(state.screen_fade_in.next()));
    }
}
