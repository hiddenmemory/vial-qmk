use alloc::vec;
use alloc::vec::Vec;
use qmk_sys::BACKLIGHT_LEVELS;

use crate::constants::{RUN_LOOP_FRAME_TIME, RUN_LOOP_START_DELAY};
use crate::display;
use crate::eeprom::EEPROM;
use crate::keyboard::Keyboard;
use crate::{
    display::Display,
    pages::{self, Page, PageState},
    sync::{SyncKey, SyncValue},
    tween::{Tween, TweenDirection},
};

pub struct State {
    pub primary_stack: Vec<Page>,
    pub secondary_stack: SyncValue<Vec<Page>>,
    pub page_layers: PageState<pages::layers::State>,
    pub page_clock: PageState<pages::clock::State>,
    pub deferred_token: u8,
    pub last_sync: u32,
    pub blue_index: SyncValue<u8>,
    pub frame_time: SyncValue<u32>,
    pub display_brightness: SyncValue<u8>,
    pub screen_fade_in: Tween<u8>,
    pub screen_fade_out: Tween<u8>,
    pub debug_output: SyncValue<bool>,
    pub date_time: SyncValue<hid_bridge::DateTime>,
}

macro_rules! get_page {
    ($state:ident , $page:expr => $($tail:tt)*) => {
        match $page {
            Page::Clock => {
                let page = &mut $state.page_clock;
                page . $($tail)*
            }
            Page::Layers => {
                let page = &mut $state.page_layers;
                page . $($tail)*
            }
        }
    };
}

#[allow(dead_code)]
impl State {
    pub fn new() -> State {
        let default_brightness = EEPROM::get_backlight();

        State {
            primary_stack: vec![Page::default_primary_page()],
            secondary_stack: SyncValue::with_fn(
                SyncKey::SecondaryDisplayStack,
                vec![Page::default_secondary_page()],
                |_, _| get().requires_redraw(),
            ),
            page_layers: pages::layers::initial(),
            page_clock: pages::clock::initial(),
            deferred_token: 0,
            last_sync: 0,
            blue_index: SyncValue::new(SyncKey::BlueDot, 0),
            frame_time: SyncValue::new(SyncKey::FrameTime, RUN_LOOP_FRAME_TIME),
            display_brightness: SyncValue::with_fn(
                SyncKey::DisplayBrightness,
                default_brightness.min(BACKLIGHT_LEVELS as u8),
                State::update_screen_brightness,
            ),
            screen_fade_in: Tween::new(0, default_brightness, default_brightness as u32 * 100)
                .delay(RUN_LOOP_START_DELAY),
            screen_fade_out: Tween::new(0, default_brightness, 0)
                .direction(TweenDirection::Backwards), // It doesn't matter duration is 0, we always set it to zero
            debug_output: SyncValue::with_fn(SyncKey::DebugOutput, true, |_, value| {
                crate::utils::debug::debug_toggle(*value);
            }),
            date_time: SyncValue::new(SyncKey::DateTime, Default::default()),
        }
    }

    fn update_screen_brightness(_previous: &u8, new: &u8) {
        let level = *new;
        let state = get();

        state.screen_fade_in.duration = level as u32 * 100;
        state.screen_fade_in.to = level;

        state.screen_fade_out.duration = level as u32 * 100;
        state.screen_fade_out.to = level;

        display::get().set_brightness(level);
    }

    pub fn incr_blue(&mut self) {
        self.blue_index
            .incr_mod(Some(qmk_sys::RGB_MATRIX_LED_COUNT as u8));
    }

    pub fn reset_screen_fade(&mut self) {
        self.screen_fade_out.finish();
        self.screen_fade_in.restart();
    }

    pub fn layout(&mut self, page: Page, display: &Display) {
        get_page!(self, page => layout(display));
    }
    pub fn update(&mut self, page: Page) {
        get_page!(self, page => update());
    }
    pub fn render(&mut self, page: Page, display: &Display) {
        get_page!(self, page => render(display));
    }

    pub fn primary_page(&self) -> Page {
        *self.primary_stack.last().unwrap()
    }

    pub fn secondary_page(&self) -> Page {
        *self.secondary_stack.get().last().unwrap()
    }

    pub fn page(&self) -> Page {
        match Keyboard::role() {
            crate::keyboard::Role::Primary => self.primary_page(),
            crate::keyboard::Role::Secondary => self.secondary_page(),
        }
    }

    pub fn other_page(&self) -> Page {
        match Keyboard::role() {
            crate::keyboard::Role::Primary => self.secondary_page(),
            crate::keyboard::Role::Secondary => self.primary_page(),
        }
    }

    pub fn requires_layout(&mut self) -> bool {
        get_page!(self, self.page() => requires_layout)
    }

    pub fn requires_redraw(&mut self) {
        if !self.primary_stack.is_empty() {
            get_page!(self, self.page() => first_render = true);
        }
    }

    pub fn push_primary_page(&mut self, page: Page) {
        self.primary_stack.push(page);
        self.requires_redraw();
    }

    pub fn pop_primary_page(&mut self) {
        self.primary_stack.pop();
        self.requires_redraw();
    }

    pub fn replace_primary_page(&mut self, page: Page) {
        self.primary_stack.pop();
        self.primary_stack.push(page);
        self.requires_redraw();
    }

    pub fn push_secondary_page(&mut self, page: Page) {
        self.secondary_stack.mutate(|inner| {
            inner.push(page);
        });
    }

    pub fn pop_secondary_page(&mut self) {
        self.secondary_stack.mutate(|inner| {
            inner.pop();
        });
    }

    pub fn replace_secondary_page(&mut self, page: Page) {
        self.secondary_stack.mutate(|inner| {
            inner.pop();
            inner.push(page);
        });
    }
}

static mut STATE: Option<State> = None;

pub fn try_get() -> Option<&'static mut State> {
    unsafe {
        #[allow(static_mut_refs)]
        STATE.as_mut()
    }
}

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
