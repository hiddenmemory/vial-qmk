use crate::{display::Display, os::HostOS, state::State, utils::HSV};

pub fn render(display: &Display, state: &mut State) {
    display.fill_rect(
        0,
        10,
        display.size.width,
        display.small_font.line_height,
        display.clear_colour,
    );

    let os = state.host_os.unwrap_or(HostOS::Other);

    super::center_text(
        display,
        &display.small_font,
        0,
        10,
        display.size.width,
        HSV::white(),
        HSV::black(),
        os.name(),
    );
}
