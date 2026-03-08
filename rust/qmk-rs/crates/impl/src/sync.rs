use crate::{
    keyboard::{Channel, Keyboard, bridge_sync, bridges},
    utils::debug_log,
};
use alloc::{boxed::Box, format, rc::Rc, vec::Vec};
use anyhow::bail;
use serde::{Deserialize, Serialize};
use spin::RwLock;

#[repr(u8)]
#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum SyncKey {
    Empty,
    Succeeded,
    Failed,
    BlueDot,
    Clock,
    DisplayPowerLevel,
    FrameTime,
    SecondarySlime,
    ScreenFade,
}

const MAGIC: u8 = 0x07;
const HEADER_LENGTH: usize = 3;

#[derive(Serialize, Deserialize, Debug)]
struct Header {
    magic: u8,
    key: SyncKey,
    body_length: u8,
}

pub trait SyncableValue: Copy + Clone + core::fmt::Debug + Eq + 'static {
    // Take a buffer and try to make the value, on success return
    // the value, oth
    fn from_wire(buf: &[u8]) -> anyhow::Result<Self>;
    // Take a buffer to write into, and return the number of bytes written
    fn to_wire(&self, buf: &mut [u8]) -> anyhow::Result<u8>;
}

pub mod syncing;

#[derive(Debug)]
pub struct SyncValue<Inner: SyncableValue> {
    key: SyncKey,
    inner: Rc<RwLock<Inner>>,
}

impl<Inner: SyncableValue> SyncValue<Inner> {
    pub fn new(key: SyncKey, value: Inner) -> SyncValue<Inner> {
        let inner = Rc::new(RwLock::new(value));

        if Keyboard::is_secondary() {
            let clone = inner.clone();

            listen(key, move |value: Inner| {
                let mut lock = clone.write();
                *lock = value;
                Ok(())
            });
        }

        SyncValue { key, inner }
    }

    pub fn with_fn<F>(key: SyncKey, value: Inner, f: F) -> SyncValue<Inner>
    where
        F: Fn(Inner) + 'static,
    {
        let inner = Rc::new(RwLock::new(value));

        if Keyboard::is_secondary() {
            let clone = inner.clone();

            listen(key, move |value: Inner| {
                {
                    let mut lock = clone.write();
                    *lock = value;
                }
                f(value);
                Ok(())
            });
        }

        SyncValue { key, inner }
    }

    pub fn set(&mut self, value: Inner) -> Inner {
        let current = { *self.inner.read() };

        if current.ne(&value) {
            {
                let mut lock = self.inner.write();
                *lock = value;
            }

            if Keyboard::is_primary() {
                if let Err(err) = send(self.key, value) {
                    debug_log(&format!("failed to sync value for {:?}: {err}", self.key))
                } else {
                    debug_log(&format!("[sync] sent {value:?} for {:?}", self.key));
                }
            }
        }

        current
    }

    pub fn get(&self) -> Inner {
        *self.inner.read()
    }

    pub fn mutate<F>(&mut self, f: F) -> Inner
    where
        F: FnOnce(&mut Inner),
    {
        let mut current = self.get();
        f(&mut current);
        self.set(current)
    }
}

type SyncTrampoline = Box<dyn Fn(&Header, &[u8]) -> anyhow::Result<()>>;

static mut SYNC_HANDLERS: Option<Vec<(SyncKey, Option<SyncTrampoline>)>> = None;

pub fn initialise() {
    if Keyboard::is_secondary() {
        unsafe {
            SYNC_HANDLERS = Some(Vec::with_capacity(16));
        }

        bridges()[Channel::AutoSync.index()].replace(Box::new(auto_sync_bridge));
    }

    debug_log("[sync] initialised");
}

fn auto_sync_bridge(
    in_len: u8,
    in_data: *const core::ffi::c_void,
    out_len: u8,
    out_data: *mut core::ffi::c_void,
) {
    unsafe {
        let incoming_data = alloc::slice::from_raw_parts(in_data as *const u8, in_len as usize);
        let outgoing_data = alloc::slice::from_raw_parts_mut(out_data as *mut u8, out_len as usize);

        if incoming_data[0] != MAGIC {
            debug_log(&format!(
                "[sync] unexpected magic {} (expecting {})",
                incoming_data[0], MAGIC
            ));
            return;
        }

        let (raw_header, raw_body) = incoming_data.split_at(HEADER_LENGTH);

        let mut header = match postcard::from_bytes::<Header>(raw_header) {
            Ok(header) => header,
            Err(err) => {
                debug_log(&format!("[sync] unable to unpack header: {err}"));
                return;
            }
        };

        let (body, _) = raw_body.split_at(header.body_length as usize);

        let handlers = {
            #[allow(static_mut_refs)]
            SYNC_HANDLERS.as_mut().unwrap()
        };

        for (potential, handler) in handlers.iter_mut() {
            if header.key.ne(potential) {
                continue;
            }

            if let Some(handler) = handler {
                match handler(&header, body) {
                    Ok(_) => {
                        header.key = SyncKey::Succeeded;
                    }
                    Err(_) => {
                        header.key = SyncKey::Failed;
                    }
                }

                header.body_length = 0;

                let _ = postcard::to_slice(&header, outgoing_data);

                return;
            }
        }
    }
}

fn listen<F, Type>(key: SyncKey, f: F)
where
    Type: SyncableValue,
    F: Fn(Type) -> anyhow::Result<()> + 'static,
{
    let inner_f = Rc::new(Box::new(f));

    let outer_f: SyncTrampoline = Box::new(move |_header: &Header, raw_body: &[u8]| {
        let body = Type::from_wire(raw_body)?;
        inner_f.as_ref()(body)
    });

    let handlers = unsafe {
        #[allow(static_mut_refs)]
        SYNC_HANDLERS.as_mut().unwrap()
    };

    // This will disable existing channels with the same message id
    // it is unlikely that there will be repeated registrations. If
    // there does become that, the handlers array can be filtered
    for (potential, handler) in handlers.iter_mut() {
        if key.eq(potential) {
            let _ = handler.take();
        }
    }

    handlers.push((key, Some(outer_f)));

    unsafe {
        qmk_sys::transaction_register_rpc(Channel::AutoSync.to_qmk_id(), Some(bridge_sync));
    }
}

fn send<Type: SyncableValue>(key: SyncKey, value: Type) -> anyhow::Result<()> {
    let mut outgoing_buffer = [0u8; 32];
    let mut incoming_buffer = [0u8; 32];

    let (header_buffer, body_buffer) = outgoing_buffer.split_at_mut(HEADER_LENGTH);

    let body_length = value.to_wire(body_buffer)?;

    let header = Header {
        magic: MAGIC,
        key,
        body_length,
    };

    postcard::to_slice(&header, header_buffer)?;

    let result = unsafe {
        qmk_sys::transaction_rpc_exec(
            Channel::AutoSync.to_qmk_id(),
            outgoing_buffer.len() as u8,
            outgoing_buffer.as_ptr() as *const core::ffi::c_void,
            incoming_buffer.len() as u8,
            incoming_buffer.as_mut_ptr() as *mut core::ffi::c_void,
        )
    };

    if result {
        match postcard::from_bytes::<Header>(&incoming_buffer) {
            Ok(header) if matches!(header.key, SyncKey::Failed) => {
                debug_log(&format!("[sync] secondary reported a failure for {key:?}"));
                bail!("unable to sync");
            }
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow::anyhow!(
                "unable to unpack header for return key {key:?}: {err}"
            )),
        }
    } else {
        Err(anyhow::anyhow!(
            "[sync] unable to communicate with secondary"
        ))
    }
}
