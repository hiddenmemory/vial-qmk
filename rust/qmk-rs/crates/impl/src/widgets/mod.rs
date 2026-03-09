use alloc::format;

use crate::{
    display::Display,
    utils::{Rect, Size},
};

pub mod clock;
pub mod image;
pub mod layer;
pub mod os;
pub mod progress;

#[derive(Copy, Clone)]
pub enum UpdateOutcome {
    RequiresRedraw,
    NoChange,
}

impl core::ops::Add for UpdateOutcome {
    type Output = UpdateOutcome;

    fn add(self, rhs: Self) -> Self::Output {
        if matches!(self, UpdateOutcome::RequiresRedraw)
            || matches!(rhs, UpdateOutcome::RequiresRedraw)
        {
            UpdateOutcome::RequiresRedraw
        } else {
            UpdateOutcome::NoChange
        }
    }
}

impl UpdateOutcome {
    pub fn requires_redraw(&self) -> bool {
        matches!(self, UpdateOutcome::RequiresRedraw)
    }
}

#[derive(Debug)]
pub struct WidgetState<Inner: Default + core::fmt::Debug> {
    pub state: Inner,
    pub requires_redraw: bool,
    pub layout_frame: Rect,
    pub ignores_accent: bool,

    pub layout_size_fn: fn(&Display, &Inner) -> Size,
    pub update_fn: fn(&mut Inner) -> UpdateOutcome,
    pub render_fn: fn(&Display, &mut Inner, Rect, bool),
}

impl<Inner: Default + core::fmt::Debug> Default for WidgetState<Inner> {
    fn default() -> Self {
        Self {
            state: Default::default(),
            requires_redraw: true,
            layout_frame: Rect::default(),
            ignores_accent: false,
            layout_size_fn: empty_size,
            update_fn: empty_update,
            render_fn: empty_render,
        }
    }
}

fn empty_size<Inner: Default + core::fmt::Debug>(_display: &Display, _state: &Inner) -> Size {
    crate::utils::debug_log(&format!(
        "[widgets] are we hooked up correctly for {_state:?}"
    ));
    Default::default()
}

fn empty_update<Inner: Default + core::fmt::Debug>(_state: &mut Inner) -> UpdateOutcome {
    UpdateOutcome::NoChange
}

fn empty_render<Inner: Default + core::fmt::Debug>(
    _display: &Display,
    _state: &mut Inner,
    _frame: Rect,
    _first_render: bool,
) {
}

impl<Inner: Default + core::fmt::Debug> WidgetState<Inner> {
    pub fn set_needs_redraw(&mut self) -> UpdateOutcome {
        self.requires_redraw = true;
        UpdateOutcome::RequiresRedraw
    }

    pub fn layout_size(&self, display: &Display) -> Size {
        (self.layout_size_fn)(display, &self.state)
    }

    pub fn update(&mut self) -> UpdateOutcome {
        let outcome = (self.update_fn)(&mut self.state);

        if outcome.requires_redraw() {
            self.set_needs_redraw();
        }

        outcome
    }

    pub fn render(&mut self, display: &Display, first_render: bool) {
        if first_render
            || self.requires_redraw
            || (!self.ignores_accent && display.accent_colour.has_changed())
        {
            (self.render_fn)(display, &mut self.state, self.layout_frame, first_render);
            self.requires_redraw = false;
        }
    }
}

#[macro_export]
macro_rules! update_widgets {
    ( $state:ident => $( $state_path:ident ), * ) => {
        {
            let mut outcome = UpdateOutcome::NoChange;
            $(
                outcome = outcome + $state. $state_path .update();
            )*

            outcome
        }
    };
}

#[macro_export]
macro_rules! render_widgets {
    ( $display:ident , $state:ident , $first_render:ident => $( $state_path:ident ),* ) => {
        {
            $(
                 $state. $state_path .render($display, $first_render);
            )*

        }
    };
}

#[macro_export]
macro_rules! layout_widgets {
    ( $display:ident , $state:ident => $( $state_path:ident ),* ) => {
        {
            let _remaining_frame = $display.bounds;
            $(
                let widget_size = $state. $state_path .layout_size($display);
                let Some((widget_frame, _remaining_frame)) = _remaining_frame.split_v(widget_size.height) else {
                    return;
                };
                $state. $state_path .layout_frame = widget_frame;

            )*

            _remaining_frame.size.height
        }
    };
}
