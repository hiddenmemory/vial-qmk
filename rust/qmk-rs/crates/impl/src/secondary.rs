use crate::{display::Display, keyboard::Keyboard, state::State, widgets};

pub fn initialise() {}

pub fn update(state: &mut State) {
    state.widget_clock.update(widgets::clock::update);

    state.widget_sleep_progress.set_progress_using(
        qmk_sys::QUANTUM_PAINTER_DISPLAY_TIMEOUT - Keyboard::last_activity_elapsed(),
        qmk_sys::QUANTUM_PAINTER_DISPLAY_TIMEOUT,
    );
}

pub fn layout(display: &Display, state: &mut State) {
    let clock_size = widgets::clock::request_size(display, &state.widget_clock.inner);

    let progress_size =
        widgets::progress::request_size(display, &state.widget_sleep_progress.inner);

    let Some((clock_frame, remaining_frame)) = display.bounds.split_v(clock_size.height) else {
        return;
    };

    let Some((progress_frame, image_frame)) = remaining_frame.split_v(progress_size.height) else {
        return;
    };

    state.widget_clock.layout_frame = clock_frame;
    state.widget_sleep_progress.layout_frame = progress_frame;
    state.widget_secondary_image.layout_frame = image_frame;
}
pub fn render(display: &mut Display, state: &mut State) {
    state.widget_clock.render(display, widgets::clock::render);

    state
        .widget_sleep_progress
        .render(display, widgets::progress::render);

    state
        .widget_secondary_image
        .render(display, widgets::image::render);
}
