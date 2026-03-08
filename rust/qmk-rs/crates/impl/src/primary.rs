use hid_bridge::MessageType;

use crate::{
    display::{self, Display},
    keyboard::Keyboard,
    keymap::KeyMap,
    rgb,
    state::{self, State},
    usb, widgets,
};

pub fn initialise() {
    if Keyboard::is_secondary() {
        return;
    }

    usb::initialise();

    usb::listen::<hid_bridge::DateTime, hid_bridge::Empty, _>(
        MessageType::DateTime,
        |_, request: Option<hid_bridge::DateTime>| {
            let state = state::get();

            if let Some(hid_bridge::DateTime {
                seconds_since_midnight,
            }) = request
            {
                state.seconds_since_midnight.set(seconds_since_midnight);
            }

            (Some(MessageType::Acknowledge), None)
        },
    );

    rgb::initialise();
}

pub fn update(state: &mut State) {
    state.widget_os.update(widgets::os::update);
    state.widget_layer.update(widgets::layer::update);
    update_image(state);

    check_screen_fades(state);
}

fn check_screen_fades(state: &mut State) {
    let display = display::get();

    if !state.screen_fade_in.finished() {
        let potential = display.get_brightness().max(state.screen_fade_in.next());

        if display.get_brightness() != potential {
            display.set_brightness(potential);
        }
    }

    // 0 means we have hit the timeout
    let timeout_diff =
        qmk_sys::QUANTUM_PAINTER_DISPLAY_TIMEOUT.saturating_sub(Keyboard::last_activity_elapsed());

    if state.screen_fade_out.finished() && timeout_diff > 0 && timeout_diff < 500 {
        state.screen_fade_out.duration = timeout_diff;
        state.screen_fade_out.reset();
    }

    if !state.screen_fade_out.finished() {
        let potential = display.get_brightness().min(state.screen_fade_out.next());

        if display.get_brightness() != potential {
            display.set_brightness(potential);
        }
    }
}

pub fn layout(display: &Display, state: &mut State) {
    let tail = display.bounds;

    let os_size = widgets::os::request_size(display, &state.widget_os.inner);
    let Some((os_frame, tail)) = tail.split_v(os_size.height) else {
        return;
    };
    state.widget_os.layout_frame = os_frame;

    let layer_size = widgets::layer::request_size(display, &state.widget_layer.inner);
    let Some((layer_frame, tail)) = tail.split_v(layer_size.height) else {
        return;
    };
    state.widget_layer.layout_frame = layer_frame;

    state.widget_primary_image.layout_frame = tail;
}

pub fn render(display: &mut Display, state: &mut State) {
    state.widget_os.render(display, widgets::os::render);

    state.widget_layer.render(display, widgets::layer::render);

    state
        .widget_primary_image
        .render(display, widgets::image::render);
}

fn update_image(state: &mut State) {
    let slime = if KeyMap::get_layer() > 0 {
        &state.orange_slime
    } else {
        &state.green_slime
    };

    state.widget_primary_image.set_image(slime);
}
