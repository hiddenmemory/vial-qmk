use crate::{display::Display, state::State};

pub fn initialise() {}

pub fn update(state: &mut State) {
    state.page_clock.update();
}

pub fn layout(display: &Display, state: &mut State) {
    state.page_clock.layout(display);
}

pub fn render(display: &mut Display, state: &mut State) {
    state.page_clock.render(display);
}
