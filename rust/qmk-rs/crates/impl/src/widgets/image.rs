use crate::{
    display::Display,
    image::Image,
    utils::{Alignment, Rect, Size},
    widgets::WidgetState,
};

#[derive(Debug, Default)]
pub struct State {
    image: Option<Image>,
    horizonal_alignment: Alignment,
    vertical_alignment: Alignment,
}

#[allow(dead_code)]
impl WidgetState<State> {
    pub fn set_image(&mut self, image: &Option<Image>) -> &mut Self {
        let existing_id = self.inner.image.as_ref().map(|image| image.id);
        let incoming_id = image.as_ref().map(|image| image.id);

        // No need to update the image
        if existing_id.eq(&incoming_id) {
            return self;
        }

        self.inner.image = *image;
        self.set_needs_display();

        self
    }

    pub fn set_vertical(&mut self, alignment: Alignment) -> &mut Self {
        self.inner.vertical_alignment = alignment;
        self
    }

    pub fn set_horizontal(&mut self, alignment: Alignment) -> &mut Self {
        self.inner.horizonal_alignment = alignment;
        self
    }
}

pub fn initial() -> WidgetState<State> {
    WidgetState {
        ignores_accent: true,
        ..Default::default()
    }
}

#[allow(dead_code)]
pub fn request_size(_display: &Display, state: &State) -> Size {
    state
        .image
        .as_ref()
        .map(|image| image.size)
        .unwrap_or_default()
}

pub fn render(display: &Display, state: &State, frame: Rect) {
    display.fill_rect(frame, *display.clear_colour);

    if let Some(image) = &state.image {
        image.draw(
            frame.position(image, state.horizonal_alignment, state.vertical_alignment),
            display,
        );
    }
}
