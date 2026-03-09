use crate::sync::SyncableValue;

impl SyncableValue for bool {
    fn from_wire(buf: &[u8]) -> anyhow::Result<Self> {
        Ok(buf[0] > 0)
    }

    fn to_wire(&self, buf: &mut [u8]) -> anyhow::Result<u8> {
        let length = 1;
        buf[0] = if *self { 1 } else { 0 };
        Ok(length)
    }
}
