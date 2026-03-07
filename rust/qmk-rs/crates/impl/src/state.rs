use serde::{Deserialize, Serialize};

use crate::{
    image::Image,
    secondary,
    widgets::{self, WidgetState},
};

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
    pub widget_os: WidgetState<widgets::os::State>,
    pub widget_layer: WidgetState<widgets::layer::State>,
    pub widget_primary_image: WidgetState<widgets::image::State>,
    pub widget_clock: WidgetState<widgets::clock::State>,
    pub widget_secondary_image: WidgetState<widgets::image::State>,
    pub deferred_token: u8,
    pub last_sync: u32,
    pub green_slime: Option<Image>,
    pub orange_slime: Option<Image>,
    // TODO this should be split into a shared state, and then we can just sync that
    // when we make changes, perhaps we have a flag to say it requires sync, then housekeeping
    // can push that change automatically to the other side
    pub secondary_slime: Slime,
    pub blue_index: u8,
}

impl State {
    pub fn new() -> State {
        State {
            backlight_level: None,
            widget_os: widgets::os::initial(),
            widget_layer: Default::default(),
            widget_primary_image: widgets::image::initial(),
            widget_clock: widgets::clock::initial(),
            widget_secondary_image: widgets::image::initial(),
            deferred_token: 0,
            last_sync: 0,
            green_slime: None,
            orange_slime: None,
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
