use crate::{display::Display, keymap::KeyMap, state::State, usb, widgets};

pub fn initialise() {
    usb::initialise();
}

pub fn update(state: &mut State) {
    state.widget_os.update(widgets::os::update);
    state.widget_layer.update(widgets::layer::update);
    update_image(state);
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
