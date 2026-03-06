use crate::{Slime, image::Image, os::HostOS};

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
            blue_index: 0
        }
    }
}

static mut STATE: State = State::new();

pub fn get() -> &'static mut State {
    unsafe {
        #[allow(static_mut_refs)]
        &mut STATE
    }
}
