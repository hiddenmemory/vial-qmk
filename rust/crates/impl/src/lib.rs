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
            if !state.primary_slime_drawn
                && let Some(ref image) = state.green_slime
            {
                image.draw(
                    display.position(image, utils::Alignment::Center, utils::Alignment::Trailing),
                    display,
                );

                state.primary_slime_drawn = true;
            }

            render_left(display, state);
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
        state.active_layer = Some(current_layer);
        widgets::layer::render(display, state);
    }
}

fn register_secondary_sync_handlers() {
    unsafe {
        qmk_sys::transaction_register_rpc(
            qmk_sys::serial_transaction_id::USER_CHANNEL_0 as i8,
            Some(secondary_sync_handler_for_rust_sync_a),
        );
    }
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
    secondary_change_slime: Option<Slime>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Default)]
struct Response {
    success: bool,
}

#[unsafe(no_mangle)]
extern "C" fn secondary_sync_handler_for_rust_sync_a(
    in_len: u8,
    in_data: *const core::ffi::c_void,
    out_len: u8,
    out_data: *mut core::ffi::c_void,
) {
    Keyboard::secondary_recv(in_len, in_data, out_len, out_data, |request: Request| {
        let success = if let Some(slime) = request.secondary_change_slime {
            let state = state::get();
            state.secondary_slime_drawn = false;
            state.secondary_slime = slime;
            true
        } else {
            false
        };

        Response { success }
    });
}

fn run_loop() {
    state::get().deferred_token =
        unsafe { qmk_sys::defer_exec(1000, Some(update), core::ptr::null_mut()) };
}

#[unsafe(no_mangle)]
pub extern "C" fn keyboard_post_init_kb_rs() {
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

    let transaction_id = qmk_sys::serial_transaction_id::USER_CHANNEL_0;

    debug_log(&format!("housekeeping {}", state.last_sync));

    state.secondary_slime = state.secondary_slime.other();

    let result: Result<Response, _> = Keyboard::seondary_send(
        transaction_id,
        Request {
            secondary_change_slime: Some(state.secondary_slime),
        },
    );

    match result {
        Ok(response) => debug_log(&format!("response = {}", response.success)),
        Err(err) => debug_log(&format!("error: {err}")),
    }
}
