#![no_std]
#![no_builtins]
#![crate_type = "staticlib"]

use serde::{Deserialize, Serialize};

extern crate alloc;
extern crate core;

use alloc::format;
#[cfg(target_arch = "arm")]
use rp2040_panic_usb_boot as _;

use crate::display::Display;
use crate::image::Image;
use crate::keyboard::{Keyboard, Role};
use crate::keymap::KeyMap;
use crate::os::HostOS;
use crate::state::State;
use crate::timer::Timer;
use crate::utils::debug_log;

mod display;
mod font;
mod heap;
mod image;
mod keyboard;
mod keymap;
mod os;
mod state;
mod timer;
mod utils;
mod widgets;

fn init(display: &mut Display, state: &mut State) {
    display.clear();
    render(display, state);
}

fn render(display: &mut Display, state: &mut State) {
    match Keyboard::role() {
        Role::Primary => {
            render_left(display, state);

            let slime = if KeyMap::get_layer() > 0 {
                &state.orange_slime
            } else {
                &state.green_slime
            };

            if !state.primary_slime_drawn
                && let Some(image) = slime
            {
                image.draw(
                    display.position(image, utils::Alignment::Center, utils::Alignment::Trailing),
                    display,
                );

                state.primary_slime_drawn = true;
            }
        }
        Role::Secondary => {
            let slime = match state.secondary_slime {
                Slime::Green => &state.green_slime,
                Slime::Orange => &state.orange_slime,
            };

            if !state.secondary_slime_drawn
                && let Some(image) = slime
            {
                image.draw(
                    display.position(image, utils::Alignment::Center, utils::Alignment::Trailing),
                    display,
                );

                state.secondary_slime_drawn = true;
            }
        }
    }
}

fn render_left(display: &mut Display, state: &mut State) {
    let current_os = HostOS::current();

    if state
        .host_os
        .map(|existing| existing.ne(&current_os))
        .unwrap_or(true)
    {
        state.host_os = Some(current_os);
        widgets::os::render(display, state);
    }

    let current_layer = KeyMap::get_layer();

    if state
        .active_layer
        .map(|existing| existing != current_layer)
        .unwrap_or(true)
    {
        state.primary_slime_drawn = false;
        state.active_layer = Some(current_layer);
        widgets::layer::render(display, state);
    }
}

fn register_secondary_sync_handlers() {
    Keyboard::listen(keyboard::Channel::A, |request: Request| {
        let state = state::get();
        state.blue_index = request.blue_index;
        state.secondary_slime_drawn = state.secondary_slime.eq(&request.secondary_change_slime);
        state.secondary_slime = request.secondary_change_slime;
        Response { success: true }
    });
}

#[derive(Default, Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Slime {
    Green,
    #[default]
    Orange,
}

impl Slime {
    fn other(&self) -> Slime {
        match self {
            Slime::Green => Slime::Orange,
            Slime::Orange => Slime::Green,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct Request {
    blue_index: u8,
    secondary_change_slime: Slime,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Default)]
struct Response {
    success: bool,
}

fn run_loop() {
    state::get().deferred_token =
        unsafe { qmk_sys::defer_exec(1000, Some(update), core::ptr::null_mut()) };
}

#[unsafe(no_mangle)]
pub extern "C" fn keyboard_post_init_rs() {
    heap::initialise();
    display::initialise();

    debug_log("initialising the display");

    let state = state::get();
    state.green_slime = unsafe { Some(Image::new(&qmk_sys::gfx_GarbageSlime)) };
    state.orange_slime = unsafe { Some(Image::new(&qmk_sys::gfx_ChefSlime)) };
    init(display::get(), state);

    debug_log("setting display brightness");
    Display::set_brightness(Display::max_brightness() / 2);

    if Keyboard::is_secondary() {
        register_secondary_sync_handlers();
    }

    debug_log("beginning run loop");
    run_loop();
}

#[unsafe(no_mangle)]
pub extern "C" fn update(_trigger_time: u32, _cb_arg: *mut core::ffi::c_void) -> u32 {
    render(display::get(), state::get());
    80 // ms
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

    state.blue_index += 1;
    if state.blue_index == qmk_sys::RGB_MATRIX_LED_COUNT as u8 {
        state.blue_index = 0;
    }

    sync_to_secondary(state);
}

fn sync_to_secondary(state: &State) {
    let result: Result<Response, _> = Keyboard::secondary_send(
        keyboard::Channel::A,
        Request {
            blue_index: state.blue_index,
            secondary_change_slime: state.secondary_slime,
        },
    );

    if let Err(err) = result {
        debug_log(&format!("error: {err}"))
    }
}

#[unsafe(no_mangle)]
/// # Safety
///
/// This is safe
pub unsafe extern "C" fn raw_hid_receive_rs(data: *mut u8, length: u8) {
    unsafe {
        let actual_data = alloc::slice::from_raw_parts_mut(data, length as usize);

        debug_log(&format!("got hid message: {:X}", actual_data[0]));

        actual_data[1] = actual_data[0];

        if actual_data[0] == 0x21 {
            let (total, used, free) = heap::usage();

            debug_log(&format!(
                "request for memory usage: total={total}, used={used}, free={free} ({}%)",
                heap::usage_percentage()
            ));
        }
    }
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
    unsafe {
        if pressed && keycode == qmk_sys::qk_keycode_defines::KC_5 as u16 {
            let state = state::get();
            state.blue_index += 1;
            sync_to_secondary(state);
            return false;
        }

        true
    }
}
