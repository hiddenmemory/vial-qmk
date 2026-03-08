use serde::{Serialize, de::DeserializeOwned};

use crate::sync::SyncableValue;

pub trait MakeSyncableValue: Copy + Clone + core::fmt::Debug + core::cmp::Eq + 'static {}

impl<T: Serialize + DeserializeOwned + MakeSyncableValue> SyncableValue for T {
    fn from_wire(buf: &[u8]) -> anyhow::Result<Self> {
        Ok(postcard::from_bytes::<T>(buf)?)
    }

    fn to_wire(&self, buf: &mut [u8]) -> anyhow::Result<u8> {
        Ok(postcard::to_slice(self, buf).map(|used| used.len() as u8)?)
    }
}
