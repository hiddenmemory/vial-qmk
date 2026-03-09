use crate::{
    display::Display,
    utils::{Rect, Size},
    widgets::{UpdateOutcome, WidgetState},
};

#[derive(Debug, Default)]
pub struct State {
    pub height: u16,
    pub padding: u16,
    pub progress: u8,
}

#[allow(dead_code)]
impl WidgetState<State> {
    pub fn set_progress(&mut self, progress: u8) -> UpdateOutcome {
        if self.state.progress != progress {
            self.state.progress = progress;
            self.set_needs_redraw()
        } else {
            UpdateOutcome::NoChange
        }
    }

    pub fn set_progress_using(&mut self, value: u32, total: u32) -> UpdateOutcome {
        let progress = ((value as f32) / (total as f32) * 100.0f32) as u8;
        self.set_progress(progress)
    }
}

pub fn initial() -> WidgetState<State> {
    let mut state: WidgetState<State> = WidgetState {
        layout_size_fn: request_size,
        render_fn: render,
        ..Default::default()
    };

    state.state.height = 4;
    state.state.padding = 2;
    state.set_progress(0);

    state
}

fn request_size(display: &Display, state: &State) -> Size {
    Size {
        width: display.bounds.size.width,
        height: state.height + (state.padding * 2),
    }
}

fn render(display: &Display, state: &mut State, frame: Rect) {
    display.fill_rect(frame, *display.clear_colour);

    let pixel_progress = ((frame.size.width as f32 / 100.0f32) * state.progress as f32) as u16;

    let stroke_frame = Rect::new(
        frame.origin.x,
        frame.origin.y + state.padding,
        frame.size.width,
        state.height,
    );

    let progress_frame = Rect::new(
        frame.origin.x,
        frame.origin.y + state.padding,
        pixel_progress,
        state.height,
    );

    display.fill_rect(progress_frame, *display.accent_colour);
    display.stroke_rect(stroke_frame, *display.accent_colour);
}
