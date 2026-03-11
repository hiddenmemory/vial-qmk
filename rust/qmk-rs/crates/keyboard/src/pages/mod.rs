use serde::{Deserialize, Serialize};

use crate::{
    display::Display,
    keyboard::{Keyboard, Role, Side},
    sync::syncing::impl_serde::MakeSyncableValue,
    widgets::Outcome,
};

pub mod clock;
pub mod layers;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Page {
    Clock,
    Layers,
}

impl Page {
    pub fn cycle(&self) -> Page {
        match self {
            Page::Clock => Page::Layers,
            Page::Layers => Page::Clock,
        }
    }

    fn default_left_page() -> Page {
        Page::Layers
    }

    fn default_right_page() -> Page {
        Page::Clock
    }

    pub fn default_primary_page() -> Page {
        match (Keyboard::role(), Keyboard::side()) {
            (Role::Primary, Side::Left) => Page::default_left_page(),
            (Role::Secondary, Side::Left) => Page::default_right_page(),
            (Role::Primary, Side::Right) => Page::default_right_page(),
            (Role::Secondary, Side::Right) => Page::default_left_page(),
        }
    }

    pub fn default_secondary_page() -> Page {
        match (Keyboard::role(), Keyboard::side()) {
            (Role::Primary, Side::Left) => Page::default_right_page(),
            (Role::Secondary, Side::Left) => Page::default_left_page(),
            (Role::Primary, Side::Right) => Page::default_left_page(),
            (Role::Secondary, Side::Right) => Page::default_right_page(),
        }
    }
}

impl MakeSyncableValue for alloc::vec::Vec<Page> {}

#[derive(Debug)]
pub struct PageState<Inner: Default + core::fmt::Debug> {
    pub state: Inner,
    pub requires_layout: bool,
    pub first_render: bool,
    pub needs_redraw: bool,
    pub ignores_accent: bool,

    pub layout_fn: fn(&Display, &mut Inner),
    pub update_fn: fn(&mut Inner) -> Outcome,
    pub render_fn: fn(&Display, &mut Inner, bool),
}

impl<Inner: Default + core::fmt::Debug> Default for PageState<Inner> {
    fn default() -> Self {
        Self {
            state: Default::default(),
            requires_layout: true,
            first_render: true,
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

fn empty_update<Inner: Default + core::fmt::Debug>(_state: &mut Inner) -> Outcome {
    Outcome::NoChange
}

fn empty_render<Inner: Default + core::fmt::Debug>(
    _display: &Display,
    _state: &mut Inner,
    _first_render: bool,
) {
}

impl<Inner: Default + core::fmt::Debug> PageState<Inner> {
    pub fn set_needs_redraw(&mut self) {
        self.needs_redraw = true;
    }

    pub fn layout(&mut self, display: &Display) {
        (self.layout_fn)(display, &mut self.state);
        self.requires_layout = false;
    }

    pub fn update(&mut self) -> Outcome {
        let outcome = (self.update_fn)(&mut self.state);

        if outcome.requires_redraw() {
            self.set_needs_redraw();
        }

        if matches!(outcome, Outcome::Layout) {
            self.requires_layout = true;
            self.first_render = true;
        }

        outcome
    }

    pub fn render(&mut self, display: &Display) {
        if self.first_render {
            display.clear();
        }

        if self.first_render
            || self.needs_redraw
            || (!self.ignores_accent && display.accent_colour.has_changed())
        {
            (self.render_fn)(display, &mut self.state, self.first_render);
            self.needs_redraw = false;
        }

        self.first_render = false;
    }
}
