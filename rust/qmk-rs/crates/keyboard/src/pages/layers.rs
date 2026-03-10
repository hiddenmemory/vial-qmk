use crate::{
    display::Display,
    keymap::KeyMap,
    layout_widgets,
    pages::PageState,
    render_widgets,
    state::Slime,
    update_widgets,
    widgets::{self, UpdateOutcome, WidgetState},
};

#[derive(Debug)]
pub struct State {
    pub widget_os: WidgetState<widgets::os::State>,
    pub widget_layer: WidgetState<widgets::layers::State>,
    pub widget_image: WidgetState<widgets::image::State>,
}

impl Default for State {
    fn default() -> Self {
        State {
            widget_os: widgets::os::initial(),
            widget_layer: widgets::layers::initial(),
            widget_image: widgets::image::initial(
                Slime::Green.image(),
                crate::utils::Alignment::Trailing,
            ),
        }
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

fn update_image(state: &mut State) -> UpdateOutcome {
    let slime = if KeyMap::get_layer() > 0 {
        Slime::Orange
    } else {
        Slime::Green
    };

    state.widget_image.set_image(slime.image())
}

pub fn update(state: &mut State) -> UpdateOutcome {
    let outcome = update_widgets! { state =>
         widget_os,
         widget_layer,
         widget_image
    };

    outcome + update_image(state)
}

pub fn layout(display: &Display, state: &mut State) {
    let remaining_height = layout_widgets! { display, state =>
         widget_os,
         widget_layer,
         widget_image
    };

    state
        .widget_image
        .layout_frame
        .origin
        .shift_v(remaining_height as i16);
}

pub fn render(display: &Display, state: &mut State, first_render: bool) {
    render_widgets! { display, state, first_render =>
        widget_os,
        widget_layer,
        widget_image
    }
}
