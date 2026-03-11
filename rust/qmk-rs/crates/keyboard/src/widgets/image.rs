use crate::{
    display::Display,
    utils::{Alignment, Rect, Size, Sizeable},
    widgets::{UpdateOutcome, WidgetState},
};

#[derive(Debug, Default)]
pub struct State {
    image: Option<&'static dyn include_image::Image>,
    horizonal_alignment: Alignment,
    vertical_alignment: Alignment,
}

#[allow(dead_code)]
impl WidgetState<State> {
    pub fn set_image(&mut self, image: &'static dyn include_image::Image) -> UpdateOutcome {
        let existing_id = self
            .state
            .image
            .as_ref()
            .map(|image| image.get_id())
            .unwrap_or(0);

        let incoming_id = image.get_id();

        // No need to update the image
        if existing_id.eq(&incoming_id) {
            return UpdateOutcome::NoChange;
        }

        self.state.image = Some(image);
        self.set_needs_redraw()
    }

    pub fn set_vertical(&mut self, alignment: Alignment) -> &mut Self {
        self.state.vertical_alignment = alignment;
        self
    }

    pub fn set_horizontal(&mut self, alignment: Alignment) -> &mut Self {
        self.state.horizonal_alignment = alignment;
        self
    }
}

pub fn initial(
    image: &'static dyn include_image::Image,
    vertical_alignment: Alignment,
) -> WidgetState<State> {
    WidgetState {
        state: State {
            image: Some(image),
            vertical_alignment,
            ..Default::default()
        },
        ignores_accent: true,
        layout_size_fn: request_size,
        render_fn: render,
        ..Default::default()
    }
}

fn request_size(_display: &Display, state: &State) -> Size {
    state
        .image
        .as_ref()
        .map(|image| image.size())
        .unwrap_or_default()
}

fn render(display: &Display, state: &mut State, frame: Rect, _first_render: bool) {

    if let Some(image) = &state.image {
        let position = frame.position(
            image.size(),
            state.horizonal_alignment,
            state.vertical_alignment,
        );

        display.render_image(position, *image, None) // Some(*display.accent_colour));
    }
}
