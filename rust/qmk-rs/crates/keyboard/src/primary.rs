use hid_bridge::MessageType;

use crate::{
    constants::SCREEN_FADE_DURATION, display::{self}, keyboard::Keyboard, rgb, state::{self, State}, timer::Timer, usb
};

pub fn initialise() {
    if Keyboard::is_secondary() {
        return;
    }

    usb::initialise();

    usb::listen::<hid_bridge::DateTime, hid_bridge::Empty, _>(
        MessageType::DateTime,
        |_, request: Option<hid_bridge::DateTime>| {
            let state = state::get();

            if let Some(hid_bridge::DateTime {
                seconds_since_midnight,
            }) = request
            {
                state.page_clock.set_seconds(seconds_since_midnight);
            }

            (Some(MessageType::Acknowledge), None)
        },
    );

    rgb::initialise();
}

pub fn check_screen_fades(state: &mut State) {
    let display = display::get();

    // 0 means we have hit the timeout
    let idle_remaining = Timer::idle_time_remaining();

    // If we are half a second to switching off the display
    if state.screen_fade_out.finished() && idle_remaining > 0 && idle_remaining < SCREEN_FADE_DURATION {
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
