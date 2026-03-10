use crate::{
    display::Display,
    image::Image,
    utils::{Alignment, Rect, Size},
    widgets::{UpdateOutcome, WidgetState},
};

#[derive(Debug, Default)]
pub struct State {
    image: Option<&'static Image>,
    horizonal_alignment: Alignment,
    vertical_alignment: Alignment,
}

#[allow(dead_code)]
impl WidgetState<State> {
    pub fn set_image(&mut self, image: &'static Image) -> UpdateOutcome {
        let existing_id = self.state.image.as_ref().map(|image| image.id).unwrap_or(0);
        let incoming_id = image.id;

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

pub fn initial(image: &'static Image, vertical_alignment: Alignment) -> WidgetState<State> {
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
        .map(|image| image.size)
        .unwrap_or_default()
}

fn render(display: &Display, state: &mut State, frame: Rect, _first_render: bool) {
    display.fill_rect(frame, *display.clear_colour);

    if let Some(image) = &state.image {
        image.draw(
            frame.position(*image, state.horizonal_alignment, state.vertical_alignment),
            display,
        );
    }
}
