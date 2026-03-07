use crate::{
    display::Display,
    os::HostOS,
    utils::{HSV, Rect, Size},
    widgets::WidgetState,
};

#[derive(Debug, Default)]
pub struct State {
    os: HostOS,
}

pub fn request_size(display: &Display, _state: &State) -> Size {
    Size {
        width: display.bounds.size.width,
        height: display.small_font.line_height,
    }
}

pub fn update(state: &mut State) -> bool {
    let current_os = HostOS::current();

    if state.os.ne(&current_os) {
        state.os = current_os;
        true
    } else {
        false
    }
}

pub fn render(display: &Display, state: &State, frame: Rect) {
    display.fill_rect(frame, display.clear_colour);

    super::center_text(
        display,
        &display.small_font,
        frame,
        HSV::white(),
        HSV::black(),
        state.os.name(),
    );
}
