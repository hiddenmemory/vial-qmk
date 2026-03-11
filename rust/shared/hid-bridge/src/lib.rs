// In `no_std` mode (i.e. when the `std` feature is absent) we suppress the
// implicit link to the standard library at the crate root.
#![cfg_attr(not(feature = "std"), no_std)]

// ---------------------------------------------------------------------------
// Feature-gate sanity checks
// ---------------------------------------------------------------------------
// These produce a clear compile error if the caller misconfigures features,
// rather than a cryptic "name not found" downstream.

#[cfg(not(any(feature = "std", feature = "no_std")))]
compile_error!(
    "hid-bridge: exactly one of the `std` or `no_std` feature flags must be \
     enabled. For bare-metal targets add `default-features = false, \
     features = [\"no_std\"]` to your dependency declaration."
);

#[cfg(all(feature = "std", feature = "no_std"))]
compile_error!("hid-bridge: the `std` and `no_std` features are mutually exclusive.");

// ---------------------------------------------------------------------------
// no_std glue
// ---------------------------------------------------------------------------
// When building without std the embedder must provide a global allocator
// (e.g. `embedded-alloc`, `dlmalloc`, or a custom `#[global_allocator]`) so
// that heap-backed types such as `String` and `Vec` remain available through
// the `alloc` crate.

#[cfg(feature = "no_std")]
extern crate alloc;

use serde::{Deserialize, Serialize};

pub const QMK_RS_CHANNEL: u8 = 0x42;
pub const QMK_RS_CHANNEL_LENGTH: usize = 10;
pub const QMK_RS_HEADER_LENGTH: usize = 5;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Ping,
    Acknowledge,
    HeapUsage,
    DateTime,
    WakeDisplays,
    ToggleDebug,
    SetFrameTime,
    QueryRgbHsv,
    SetRgbHsv,
    SetDisplayBrightness,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageHeader {
    pub magic: u8,
    pub message_type: MessageType,
    pub packet_length: u8,
    pub _reserved_packet_counter: u8,
    pub _reserved_packet_total: u8,
}

impl MessageHeader {
    pub fn new(message_type: MessageType, packet_length: u8) -> MessageHeader {
        MessageHeader {
            magic: QMK_RS_CHANNEL,
            message_type,
            packet_length,
            _reserved_packet_counter: 1,
            _reserved_packet_total: 1,
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct DateTime {
    /// Current time — seconds elapsed since local midnight right now.
    pub seconds_since_midnight: u32,
    /// Civil dawn — sun is 6° below horizon; usable outdoor light begins.
    pub dawn_secs: u32,
    /// Sunrise — upper limb of the sun crosses the horizon.
    pub sunrise_secs: u32,
    /// Sunset — upper limb of the sun drops below the horizon.
    pub sunset_secs: u32,
    /// Civil dusk — sun is 6° below horizon; usable outdoor light ends.
    pub dusk_secs: u32,
}

impl DateTime {
    pub fn usable(&self) -> bool {
        self.seconds_since_midnight > 0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoolValue {
    pub value: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct U32Value {
    pub value: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HsvValue {
    pub h: u16,
    pub s: u8,
    pub v: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct U8Value {
    pub value: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct U8ValueWithFlag {
    pub value: u8,
    pub flag: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Empty;
