use crate::{
    display::Display,
    os::HostOS,
    utils::{HSV_BLACK, HSV_WHITE, Rect, Size},
    widgets::{UpdateOutcome, WidgetState},
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
        height: display.small_font.line_height,
    }
}

fn update(state: &mut State) -> UpdateOutcome {
    let current_os = HostOS::current();

    if state.os.ne(&current_os) {
        state.os = current_os;
        return UpdateOutcome::RequiresRedraw;
    }

    UpdateOutcome::NoChange
}

fn render(display: &Display, state: &mut State, frame: Rect, _first_render: bool) {
    display.fill_rect(frame, *display.clear_colour);

    display.center_text(
        &display.small_font,
        frame,
        HSV_WHITE,
        HSV_BLACK,
        state.os.name(),
    );
}
