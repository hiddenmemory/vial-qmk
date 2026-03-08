use anyhow::bail;

use crate::{display::PowerLevel, sync::SyncableValue};

impl SyncableValue for PowerLevel {
    fn from_wire(buf: &[u8]) -> anyhow::Result<Self> {
        if buf.len() < 2 {
            bail!("[PowerLevel.from_wire] expected a buffer with at least 2 bytes");
        }

        Ok(match buf[0] {
            0 => PowerLevel::Off(buf[1]),
            _ => PowerLevel::On(buf[1]),
        })
    }

    fn to_wire(&self, buf: &mut [u8]) -> anyhow::Result<u8> {
        let length = 2;

        if buf.len() < 2 {
            bail!("[PowerLevel.to_wire] expected a buffer with at least 2 bytes");
        }

        buf[0] = if self.is_off() { 0 } else { 1 };
        buf[1] = self.level();

        Ok(length)
    }
}
