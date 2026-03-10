use crate::sync::{SyncValue, SyncableValue};

impl SyncableValue for u8 {
    fn from_wire(buf: &[u8]) -> anyhow::Result<Self> {
        Ok(buf[0])
    }

    fn to_wire(&self, buf: &mut [u8]) -> anyhow::Result<u8> {
        buf[0] = *self;
        Ok(1)
    }
}

impl SyncValue<u8> {
    pub fn incr_mod(&mut self, modulo: Option<u8>) {
        self.set(self.get().wrapping_add(1) % modulo.unwrap_or(255));
    }
}
