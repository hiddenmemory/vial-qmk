use anyhow::bail;

use crate::sync::SyncableValue;

impl SyncableValue for u32 {
    fn from_wire(buf: &[u8]) -> anyhow::Result<Self> {
        if buf.len() < 4 {
            bail!("[u32.from_wire] expected a buffer with at least 4 bytes");
        }

        Ok((((buf[0] as u32) << 24) & 0xFF000000)
            + (((buf[1] as u32) << 16) & 0x00FF0000)
            + (((buf[2] as u32) << 8) & 0x0000FF00)
            + ((buf[3] as u32) & 0x000000FF))
    }

    fn to_wire(&self, buf: &mut [u8]) -> anyhow::Result<u8> {
        let length = 4;

        if buf.len() < 4 {
            bail!("[u32.to_wire] expected a buffer with at least 4 bytes");
        }

        buf[0] = ((self >> 24) as u8) & 0xFF;
        buf[1] = ((self >> 16) as u8) & 0xFF;
        buf[2] = ((self >> 8) as u8) & 0xFF;
        buf[3] = (*self as u8) & 0xFF;

        Ok(length)
    }
}
