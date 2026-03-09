use crate::{display::Display, widgets::UpdateOutcome};

pub mod clock;
pub mod layers;

#[derive(Debug)]
pub struct PageState<Inner: Default + core::fmt::Debug> {
    pub state: Inner,
    pub needs_redraw: bool,
    pub ignores_accent: bool,

    pub layout_fn: fn(&Display, &mut Inner),
    pub update_fn: fn(&mut Inner) -> UpdateOutcome,
    pub render_fn: fn(&Display, &mut Inner),
}

impl<Inner: Default + core::fmt::Debug> Default for PageState<Inner> {
    fn default() -> Self {
        Self {
            state: Default::default(),
            needs_redraw: true,
            ignores_accent: false,
            layout_fn: empty_layout,
            update_fn: empty_update,
            render_fn: empty_render,
        }
    }
}

fn empty_layout<Inner: Default + core::fmt::Debug>(_display: &Display, _state: &mut Inner) {
    Default::default()
}

fn empty_update<Inner: Default + core::fmt::Debug>(_state: &mut Inner) -> UpdateOutcome {
    UpdateOutcome::NoChange
}

fn empty_render<Inner: Default + core::fmt::Debug>(_display: &Display, _state: &mut Inner) {}

impl<Inner: Default + core::fmt::Debug> PageState<Inner> {
    pub fn set_needs_redraw(&mut self) {
        self.needs_redraw = true;
    }

    pub fn layout(&mut self, display: &Display) {
        (self.layout_fn)(display, &mut self.state);
    }

    pub fn update(&mut self) -> UpdateOutcome {
        let outcome = (self.update_fn)(&mut self.state);

        if outcome.requires_redraw() {
            self.set_needs_redraw();
        }

        outcome
    }

    pub fn render(&mut self, display: &Display) {
        if self.needs_redraw || (!self.ignores_accent && display.accent_colour.has_changed()) {
            (self.render_fn)(display, &mut self.state);
            self.needs_redraw = false;
        }
    }
}
