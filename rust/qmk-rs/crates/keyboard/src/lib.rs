#![no_std]
#![no_builtins]
#![crate_type = "staticlib"]

extern crate alloc;
extern crate core;

#[cfg(target_arch = "arm")]
use rp2040_panic_usb_boot as _;

use crate::constants::RUN_LOOP_START_DELAY;
use crate::display::Display;
use crate::keyboard::Keyboard;
use crate::keymap::KeyMap;
use crate::state::State;
use crate::timer::Timer;
use crate::utils::debug::{self, debug_log};
use crate::utils::{HSV_LIME, HSV_ORANGE};

mod constants;
mod detect_os;
mod display;
mod font;
mod heap;
mod image;
mod keyboard;
mod keymap;
mod pages;
mod primary;
mod rgb;
mod secondary;
mod state;
mod sync;
mod timer;
mod tween;
mod usb;
mod utils;
mod widgets;
mod eeprom;

fn render_frame() {
    let state = state::get();
    let display = display::get();

    update(state);
    render(display, state);
}

fn update(state: &mut State) {
    state.update(state.page());
}

fn render(display: &mut Display, state: &mut State) {
    if KeyMap::get_layer() > 0 {
        display.accent_colour.set(HSV_ORANGE);
    } else {
        display.accent_colour.set(HSV_LIME);
    }

    if state.requires_layout() {
        state.layout(state.page(), display);
    }

    state.render(state.page(), display);

    // display.test();
    display.flush();
}

fn run_loop() {
    state::get().deferred_token = unsafe {
        qmk_sys::defer_exec(
            RUN_LOOP_START_DELAY,
            Some(if Keyboard::is_primary() {
                run_loop_primary
            } else {
                run_loop_secondary
            }),
            core::ptr::null_mut(),
        )
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn keyboard_post_init_rs() {
    heap::initialise();
    debug::initialise();
    sync::initialise();

    debug_log("initialising the display");
    let display = display::initialise();

    debug_log("setting display brightness");
    display.set_brightness(0);
    display.clear();

    debug_log("setting up initial state");
    state::initialise();

    debug_log("applying first render");
    render_frame();

    debug_log("setting up side specific configuration");
    if Keyboard::is_primary() {
        primary::initialise();
    } else {
        secondary::initialise();
    }

    debug_log("beginning run loop");
    run_loop();

    display.set_brightness(0);
}

#[unsafe(no_mangle)]
pub extern "C" fn run_loop_primary(_trigger_time: u32, _cb_arg: *mut core::ffi::c_void) -> u32 {
    let displays_off = Keyboard::last_activity_elapsed() > qmk_sys::QUANTUM_PAINTER_DISPLAY_TIMEOUT;

    if displays_off {
        display::get().assume_off();
    } else {
        render_frame();
    }

    state::get().frame_time.get() // ms
}

#[unsafe(no_mangle)]
pub extern "C" fn run_loop_secondary(_trigger_time: u32, _cb_arg: *mut core::ffi::c_void) -> u32 {
    if !display::get().power_level.get().is_off() {
        render_frame();
    }

    state::get().frame_time.get() // ms
}

#[unsafe(no_mangle)]
pub extern "C" fn housekeeping_task_user_rs() {
    let state = state::get();

    if Keyboard::is_primary() {
        utils::debug::check_secondary_debug_queue();
        primary::check_screen_fades(state);
    }

    let elapsed = Timer::elapsed(state.last_sync);

    if elapsed < 2000 {
        return;
    }

    state.last_sync = Timer::read();

    if Keyboard::is_primary() {
        state.incr_blue();
    }
}

#[unsafe(no_mangle)]
/// # Safety
///
/// This is safe
pub unsafe extern "C" fn process_record_user_rs(
    keycode: u16,
    pressed: bool,
    _record: *const qmk_sys::keyrecord_t,
) -> bool {
    if !Keyboard::is_primary() {
        return true;
    }

    check_display_state_and_render();

    if pressed && keycode == qmk_sys::qk_keycode_defines::KC_5 as u16 {
        state::get().incr_blue();
        return false;
    }

    if pressed && keycode == qmk_sys::qk_keycode_defines::KC_4 as u16 {
        let state = state::get();
        state.replace_primary_page(state.page().cycle());
        state.replace_secondary_page(state.other_page().cycle());
    }

    true
}

pub fn check_display_state_and_render() {
    let display = display::get();
    let state = state::get();

    if display.power_level.get().is_off() || state.screen_fade_out.running() {
        state.reset_screen_fade();
        display.assume_on();
    }
}
