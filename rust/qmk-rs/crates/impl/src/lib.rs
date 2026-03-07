#![no_std]
#![no_builtins]
#![crate_type = "staticlib"]

use serde::{Deserialize, Serialize};

extern crate alloc;
extern crate core;

#[cfg(target_arch = "arm")]
use rp2040_panic_usb_boot as _;

use crate::display::Display;
use crate::image::Image;
use crate::keyboard::{Keyboard, Role};
use crate::keymap::KeyMap;
use crate::state::{Slime, State};
use crate::timer::Timer;
use crate::utils::debug_log;

mod display;
mod font;
mod heap;
mod image;
mod keyboard;
mod keymap;
mod os;
mod primary;
mod secondary;
mod state;
mod timer;
mod usb;
mod utils;
mod widgets;

fn first_render(display: &mut Display, state: &mut State) {
    display.clear();
    render(display, state);
}

fn update_state(state: &mut State) {
    match Keyboard::role() {
        Role::Primary => primary::update(state),
        Role::Secondary => secondary::update(state),
    }
}

fn layout(display: &mut Display, state: &mut State) {
    match Keyboard::role() {
        Role::Primary => primary::layout(display, state),
        Role::Secondary => secondary::layout(display, state),
    }
}

fn render(display: &mut Display, state: &mut State) {
    match Keyboard::role() {
        Role::Primary => primary::render(display, state),
        Role::Secondary => secondary::render(display, state),
    }
    display.flush();
}

fn run_loop() {
    state::get().deferred_token =
        unsafe { qmk_sys::defer_exec(1000, Some(update), core::ptr::null_mut()) };
}

#[unsafe(no_mangle)]
pub extern "C" fn keyboard_post_init_rs() {
    heap::initialise();

    debug_log("initialising the display");
    display::initialise();

    debug_log("setting display brightness");
    Display::set_brightness(Display::max_brightness() / 3);

    debug_log("setting up initial state");
    let state = state::initialise(initialise_state);

    debug_log("applying first render");
    update_state(state);
    layout(display::get(), state);
    first_render(display::get(), state);

    debug_log("setting up side specific configuration");
    if Keyboard::is_primary() {
        primary::initialise();
    } else {
        secondary::initialise();
    }

    debug_log("beginning run loop");
    run_loop();
}

fn initialise_state(state: &mut State) {
    state.green_slime = unsafe { Some(Image::new(&qmk_sys::gfx_GarbageSlime)) };
    state.orange_slime = unsafe { Some(Image::new(&qmk_sys::gfx_ChefSlime)) };

    state
        .widget_primary_image
        .set_image(&state.green_slime)
        .set_vertical(utils::Alignment::Trailing);

    state
        .widget_secondary_image
        .set_image(&state.orange_slime)
        .set_vertical(utils::Alignment::Trailing);
}

#[unsafe(no_mangle)]
pub extern "C" fn update(_trigger_time: u32, _cb_arg: *mut core::ffi::c_void) -> u32 {
    let state = state::get();

    update_state(state);
    render(display::get(), state);

    32 // ms
}

#[unsafe(no_mangle)]
pub extern "C" fn housekeeping_task_user_rs() {
    if !Keyboard::is_primary() {
        return;
    }

    let state = state::get();
    let elapsed = Timer::elapsed(state.last_sync);

    if elapsed < 2000 {
        return;
    }

    state.last_sync = Timer::read();
    state.secondary_slime = state.secondary_slime.other();
    state.incr_blue();

    secondary::sync(state);
}

#[unsafe(no_mangle)]
pub extern "C" fn rgb_matrix_indicators_advanced_rs(min: u8, max: u8) {
    unsafe {
        let state = state::get();
        let value = 0x05;

        for offset in min..=max {
            match KeyMap::get_layer() {
                _ if offset == state.blue_index => {
                    qmk_sys::rgb_matrix_set_color(offset as i32, 0x00, value, value)
                }
                0 if matches!(Keyboard::role(), Role::Primary)
                    || matches!(state.secondary_slime, Slime::Green) =>
                {
                    qmk_sys::rgb_matrix_set_color(offset as i32, 0x00, value, 0x00)
                }
                _ => qmk_sys::rgb_matrix_set_color(offset as i32, value, value / 2, 0),
            }
        }
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
    if pressed && keycode == qmk_sys::qk_keycode_defines::KC_5 as u16 {
        state::get().incr_blue().sync();
        return false;
    }

    true
}
