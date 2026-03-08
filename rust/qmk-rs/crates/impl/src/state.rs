use serde::{Deserialize, Serialize};

use crate::{
    image::Image,
    sync::{SyncKey, SyncValue, syncing::impl_serde::MakeSyncableValue},
    widgets::{self, WidgetState},
};

#[derive(Default, Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Slime {
    Green,
    #[default]
    Orange,
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
    pub widget_os: WidgetState<widgets::os::State>,
    pub widget_layer: WidgetState<widgets::layer::State>,
    pub widget_primary_image: WidgetState<widgets::image::State>,
    pub widget_clock: WidgetState<widgets::clock::State>,
    pub widget_sleep_progress: WidgetState<widgets::progress::State>,
    pub widget_secondary_image: WidgetState<widgets::image::State>,
    pub deferred_token: u8,
    pub last_sync: u32,
    pub seconds_since_midnight: SyncValue<u32>,
    pub green_slime: Option<Image>,
    pub orange_slime: Option<Image>,
    // TODO this should be split into a shared state, and then we can just sync that
    // when we make changes, perhaps we have a flag to say it requires sync, then housekeeping
    // can push that change automatically to the other side
    pub secondary_slime: SyncValue<Slime>,
    pub blue_index: SyncValue<u8>,
    pub frame_time: SyncValue<u32>,
}

impl State {
    pub fn new() -> State {
        State {
            widget_os: widgets::os::initial(),
            widget_layer: Default::default(),
            widget_primary_image: widgets::image::initial(),
            widget_clock: widgets::clock::initial(),
            widget_sleep_progress: widgets::progress::initial(),
            widget_secondary_image: widgets::image::initial(),
            deferred_token: 0,
            last_sync: 0,
            seconds_since_midnight: SyncValue::with_fn(SyncKey::Clock, 0, |value| {
                get().widget_clock.set_seconds(value);
            }),
            green_slime: None,
            orange_slime: None,
            secondary_slime: SyncValue::with_fn(SyncKey::SecondarySlime, Slime::Orange, |slime| {
                let state = get();

                let image = match slime {
                    Slime::Green => &state.green_slime,
                    Slime::Orange => &state.orange_slime,
                };

                state.widget_secondary_image.set_image(image);
            }),
            blue_index: SyncValue::new(SyncKey::BlueDot, 0),
            frame_time: SyncValue::new(SyncKey::FrameTime, 32),
        }
    }

    pub fn incr_blue(&mut self) {
        self.blue_index
            .incr_mod(Some(qmk_sys::RGB_MATRIX_LED_COUNT as u8));
    }
}

static mut STATE: Option<State> = None;

pub fn get() -> &'static mut State {
    unsafe {
        #[allow(static_mut_refs)]
        STATE.as_mut().unwrap()
    }
}

pub fn initialise<F>(f: F) -> &'static mut State
where
    F: Fn(&mut State),
{
    unsafe {
        #[allow(static_mut_refs)]
        if STATE.is_none() {
            STATE = Some(State::new())
        }

        #[allow(static_mut_refs)]
        let state = STATE.as_mut().unwrap();
        f(state);
        state
    }
}
