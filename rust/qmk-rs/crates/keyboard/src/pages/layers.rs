use crate::{
    display::Display,
    keymap::KeyMap,
    layout_widgets,
    pages::PageState,
    render_widgets, update_widgets,
    utils::Rect,
    widgets::{self, Outcome, WidgetState},
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
                &crate::images::GREEN,
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

fn update_image(state: &mut State) -> Outcome {
    let image: &dyn include_image::Image = if KeyMap::get_layer() > 0 {
        &crate::images::ORANGE
    } else {
        &crate::images::GREEN
    };

    let outcome = state.widget_image.set_image(image);

    if outcome.requires_redraw() {
        state.widget_layer.set_needs_redraw();
    }

    outcome
}

pub fn update(state: &mut State) -> Outcome {
    let outcome = update_widgets! { state =>
         widget_os,
         widget_layer,
         widget_image
    };

    outcome + update_image(state)
}

pub fn layout(display: &Display, state: &mut State) {
    layout_widgets! { display, state =>
         widget_os,
         widget_layer
    };

    let size = state.widget_image.layout_size(display);

    state.widget_image.layout_frame = Rect::new(
        0,
        display.bounds.size.height - size.height,
        size.width,
        size.height,
    );
}

pub fn render(display: &mut Display, state: &mut State, first_render: bool) {
    if state.widget_layer.requires_redraw && state.widget_image.requires_redraw {
        display.fill_rect(state.widget_image.layout_frame, *display.clear_colour);
    }

    render_widgets! { display, state, first_render =>
        widget_os,
        widget_layer,
        widget_image
    }
}
