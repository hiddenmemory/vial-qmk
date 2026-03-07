use serde::{Deserialize, Serialize};

use crate::{image::Image, os::HostOS, secondary};

#[derive(Default, Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Slime {
    Green,
    #[default]
    Orange,
}

impl Slime {
    pub fn other(&self) -> Slime {
        match self {
            Slime::Green => Slime::Orange,
            Slime::Orange => Slime::Green,
        }
    }
}

#[derive(Default)]
pub struct State {
    pub backlight_level: Option<u8>,
    pub active_layer: Option<u8>,
    pub host_os: Option<HostOS>,
    pub deferred_token: u8,
    pub last_sync: u32,
    pub green_slime: Option<Image>,
    pub orange_slime: Option<Image>,
    pub primary_slime_drawn: bool,
    pub secondary_slime_drawn: bool,
    // TODO this should be split into a shared state, and then we can just sync that
    // when we make changes, perhaps we have a flag to say it requires sync, then housekeeping
    // can push that change automatically to the other side
    pub secondary_slime: Slime,
    pub blue_index: u8,
}

impl State {
    pub const fn new() -> State {
        State {
            backlight_level: None,
            active_layer: None,
            host_os: None,
            deferred_token: 0,
            last_sync: 0,
            green_slime: None,
            orange_slime: None,
            primary_slime_drawn: false,
            secondary_slime_drawn: false,
            secondary_slime: Slime::Orange,
            blue_index: 0,
        }
    }

    pub fn incr_blue(&mut self) -> &mut State {
        self.blue_index += 1;

        if self.blue_index == qmk_sys::RGB_MATRIX_LED_COUNT as u8 {
            self.blue_index = 0;
        }

        self
    }

    pub fn sync(&self) {
        secondary::sync(self);
    }
}

static mut STATE: State = State::new();

pub fn get() -> &'static mut State {
    unsafe {
        #[allow(static_mut_refs)]
        &mut STATE
    }
}
