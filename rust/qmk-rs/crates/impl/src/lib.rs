#![no_std]
#![no_builtins]
#![crate_type = "staticlib"]

extern crate alloc;
extern crate core;

#[cfg(target_arch = "arm")]
use rp2040_panic_usb_boot as _;

use crate::display::Display;
use crate::image::Image;
use crate::keyboard::{Keyboard, Role};
use crate::keymap::KeyMap;
use crate::state::State;
use crate::timer::Timer;
use crate::utils::{HSV, debug_log};

mod display;
mod font;
mod heap;
mod image;
mod keyboard;
mod keymap;
mod os;
mod primary;
mod rgb;
mod secondary;
mod state;
mod sync;
mod timer;
mod usb;
mod utils;
mod widgets;

fn first_render(display: &mut Display, state: &mut State) {
    display.clear();
    render(display, state);
}

fn render_frame() {
    let state = state::get();
    update_state(state);
    render(display::get(), state);
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
    if KeyMap::get_layer() > 0 {
        display.accent_colour.set(HSV::papaya());
    } else {
        display.accent_colour.set(HSV::paulo());
    }

    match Keyboard::role() {
        Role::Primary => primary::render(display, state),
        Role::Secondary => secondary::render(display, state),
    }
    display.flush();
}

fn run_loop() {
    state::get().deferred_token = unsafe {
        qmk_sys::defer_exec(
            1000,
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
    sync::initialise();

    debug_log("initialising the display");
    let display = display::initialise();

    debug_log("setting display brightness");
    display.set_brightness(Display::max_brightness() / 2);

    debug_log("setting up initial state");
    let state = state::initialise(initialise_state);

    debug_log("applying first render");
    update_state(state);
    layout(display, state);
    first_render(display, state);

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
/// # Safety
///
/// This is safe
pub unsafe extern "C" fn process_record_user_rs(
    keycode: u16,
    pressed: bool,
    _record: *const qmk_sys::keyrecord_t,
) -> bool {
    check_display_state_and_render();

    if pressed && keycode == qmk_sys::qk_keycode_defines::KC_5 as u16 {
        state::get().incr_blue();
        return false;
    }

    true
}

pub fn check_display_state_and_render() {
    let should_render_frame = {
        let display = display::get();

        if display.power_level.get().is_off() {
            display::get().assume_on();
            true
        } else {
            false
        }
    };

    if should_render_frame {
        debug_log("forcing frame render");
        render_frame();
    }
}
