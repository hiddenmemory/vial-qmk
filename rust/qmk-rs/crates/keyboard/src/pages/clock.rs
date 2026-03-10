use crate::{
    display::Display,
    layout_widgets,
    pages::PageState,
    render_widgets,
    state::{self},
    sync::{SyncKey, SyncValue},
    timer::Timer,
    update_widgets,
    widgets::{self, UpdateOutcome, WidgetState},
};

#[derive(Debug)]
pub struct State {
    pub widget_clock: WidgetState<widgets::clock::State>,
    pub widget_sleep_progress: WidgetState<widgets::progress::State>,
    pub widget_huge_layer: WidgetState<widgets::huge_layer::State>,
    pub seconds_since_midnight: SyncValue<u32>,
}

impl Default for State {
    fn default() -> Self {
        State {
            widget_clock: widgets::clock::initial(),
            widget_sleep_progress: widgets::progress::initial(),
            widget_huge_layer: widgets::huge_layer::initial(),
            seconds_since_midnight: SyncValue::with_fn(SyncKey::Clock, 0, |_, value| {
                state::get()
                    .page_clock
                    .state
                    .widget_clock
                    .set_seconds(*value);
            }),
        }
    }
}

impl PageState<State> {
    pub fn set_seconds(&mut self, seconds: u32) {
        self.state.seconds_since_midnight.set(seconds);
    }
}

pub fn initial() -> PageState<State> {
    PageState {
        layout_fn: layout,
        update_fn: update,
        render_fn: render,
        ..Default::default()
    }
}

pub fn update(state: &mut State) -> UpdateOutcome {
    let outcome = update_widgets! { state =>
        widget_clock,
        widget_huge_layer
    };

    let sleep_outcome = state.widget_sleep_progress.set_progress_using(
        Timer::idle_time_remaining(),
        qmk_sys::QUANTUM_PAINTER_DISPLAY_TIMEOUT,
    );

    outcome + sleep_outcome
}

pub fn layout(display: &Display, state: &mut State) {
    let remaining_height = layout_widgets! { display, state =>
        widget_clock,
        widget_sleep_progress,
        widget_huge_layer
    };

    state.widget_huge_layer.layout_frame.size.height += remaining_height;
}

pub fn render(display: &Display, state: &mut State, first_render: bool) {
    render_widgets! { display, state, first_render =>
        widget_clock,
        widget_sleep_progress,
        widget_huge_layer
    }
}
