use serde::{Deserialize, Serialize};

use crate::{
    display::Display,
    image::{GREEN_SLIME, Image, ORANGE_SLIME},
    pages::{self, PageState},
    sync::{SyncKey, SyncValue, syncing::impl_serde::MakeSyncableValue},
    tween::{Tween, TweenDirection},
};

pub const RUN_LOOP_START_DELAY: u32 = 1000;

#[derive(Default, Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Slime {
    Green,
    #[default]
    Orange,
}

impl Slime {
    #[inline]
    pub fn image(&self) -> &'static Image {
        match self {
            Slime::Green => &GREEN_SLIME,
            Slime::Orange => &ORANGE_SLIME,
        }
    }
}

impl MakeSyncableValue for Slime {}

impl Slime {
    pub fn other(&self) -> Slime {
        match self {
            Slime::Green => Slime::Orange,
            Slime::Orange => Slime::Green,
        }
    }
}

pub struct State {
    pub page_layers: PageState<pages::layers::State>,
    pub page_clock: PageState<pages::clock::State>,
    pub deferred_token: u8,
    pub last_sync: u32,
    pub blue_index: SyncValue<u8>,
    pub frame_time: SyncValue<u32>,
    pub screen_fade_in: Tween<u8>,
    pub screen_fade_out: Tween<u8>,
}

impl State {
    pub fn new() -> State {
        State {
            page_layers: pages::layers::initial(),
            page_clock: pages::clock::initial(),
            deferred_token: 0,
            last_sync: 0,
            blue_index: SyncValue::new(SyncKey::BlueDot, 0),
            frame_time: SyncValue::new(SyncKey::FrameTime, 32),
            screen_fade_in: Tween::new(0, Display::max_brightness() / 2 + 1, 500)
                .delay(RUN_LOOP_START_DELAY),
            screen_fade_out: Tween::new(0, Display::max_brightness() / 2 + 1, 0)
                .direction(TweenDirection::Backwards), // It doesn't matter duration is 0, we always set it to zero
        }
    }

    pub fn incr_blue(&mut self) {
        self.blue_index
            .incr_mod(Some(qmk_sys::RGB_MATRIX_LED_COUNT as u8));
    }

    pub fn reset_screen_fade(&mut self) {
        self.screen_fade_out.finish();
        self.screen_fade_in.restart();
    }
}

static mut STATE: Option<State> = None;

pub fn get() -> &'static mut State {
    unsafe {
        #[allow(static_mut_refs)]
        STATE.as_mut().unwrap()
    }
}

pub fn initialise() -> &'static mut State {
    unsafe {
        #[allow(static_mut_refs)]
        if STATE.is_none() {
            STATE = Some(State::new())
        }

        #[allow(static_mut_refs)]
        let state = STATE.as_mut().unwrap();
        state
    }
}
