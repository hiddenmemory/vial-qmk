use alloc::format;
use serde::{Deserialize, Serialize};

use crate::{
    keyboard::{Channel, Keyboard},
    state::{self, Slime, State},
    utils::debug_log,
};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct SyncStateRequest {
    blue_index: u8,
    secondary_change_slime: Slime,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Default)]
struct SyncStateResponse {
    success: bool,
}

pub fn initialise() {
    if Keyboard::is_primary() {
        return;
    }

    Keyboard::listen(Channel::A, |request: SyncStateRequest| {
        let state = state::get();
        state.blue_index = request.blue_index;
        state.secondary_slime_drawn = state.secondary_slime.eq(&request.secondary_change_slime);
        state.secondary_slime = request.secondary_change_slime;
        SyncStateResponse { success: true }
    });
}

pub fn sync(state: &State) {
    if Keyboard::is_secondary() {
        return;
    }

    let result: Result<SyncStateResponse, _> = Keyboard::send(
        Channel::A,
        SyncStateRequest {
            blue_index: state.blue_index,
            secondary_change_slime: state.secondary_slime,
        },
    );

    if let Err(err) = result {
        debug_log(&format!("error: {err}"))
    }
}
