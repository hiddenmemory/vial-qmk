use crate::{
    detect_os::HostOS,
    display::Display,
    fonts,
    utils::{Alignment, HSV_BLACK, HSV_WHITE, Rect, Size},
    widgets::{Outcome, WidgetState},
};

#[derive(Debug, Default)]
pub struct State {
    os: HostOS,
}

pub fn initial() -> WidgetState<State> {
    WidgetState {
        layout_size_fn: request_size,
        update_fn: update,
        render_fn: render,
        ignores_accent: true,
        ..Default::default()
    }
}

fn request_size(display: &Display, _state: &State) -> Size {
    Size {
        width: display.bounds.size.width,
        height: fonts::SMALL.height as u16,
    }
}

fn update(state: &mut State) -> Outcome {
    let current_os = HostOS::current();

    if state.os.ne(&current_os) {
        state.os = current_os;
        return Outcome::Redraw;
    }

    Outcome::NoChange
}

fn render(display: &mut Display, state: &mut State, frame: Rect, _first_render: bool) {
    display.fill_rect(frame, *display.clear_colour);

    fonts::render_aligned(
        display,
        &fonts::SMALL,
        frame,
        Alignment::Center,
        HSV_WHITE,
        Some(HSV_BLACK),
        state.os.name(),
    );
}
