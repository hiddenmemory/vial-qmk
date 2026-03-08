use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub enum Side {
    Left,
    Right,
}

pub enum Role {
    Primary,
    Secondary,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum Channel {
    A,
    Sync,
    C,
    D,
    E,
    F,
}

pub struct Keyboard;

#[allow(dead_code)]
impl Keyboard {
    #[inline]
    pub fn side() -> Side {
        if Self::is_lhs() {
            Side::Left
        } else {
            Side::Right
        }
    }
    #[inline]
    pub fn is_lhs() -> bool {
        unsafe { qmk_sys::is_keyboard_left() }
    }
    #[inline]
    pub fn is_rhs() -> bool {
        !Self::is_lhs()
    }
    #[inline]
    pub fn role() -> Role {
        if Self::is_primary() {
            Role::Primary
        } else {
            Role::Secondary
        }
    }
    #[inline]
    pub fn is_primary() -> bool {
        unsafe { qmk_sys::is_keyboard_master() }
    }
    #[inline]
    pub fn is_secondary() -> bool {
        !Self::is_primary()
    }

    pub fn trigger_fake_activity() {
        unsafe {
            unsafe extern "C" {
                pub fn last_matrix_activity_trigger() -> bool;
            }

            last_matrix_activity_trigger();
        }
    }

    pub fn last_activity() -> u32 {
        unsafe { qmk_sys::last_input_activity_time() }
    }

    pub fn last_activity_elapsed() -> u32 {
        unsafe { qmk_sys::last_input_activity_elapsed() }
    }

    pub fn send<Request: Serialize, Response: Default + Serialize + DeserializeOwned>(
        channel: Channel,
        request: Request,
    ) -> Result<Response> {
        let request_buffer =
            postcard::to_allocvec(&request).context("unable to serialize request")?;

        let response_template = Response::default();
        let mut response_buffer = postcard::to_allocvec(&response_template)
            .context("unable to serialize response template")?;

        let result = unsafe {
            qmk_sys::transaction_rpc_exec(
                channel.to_qmk_id(),
                request_buffer.len() as u8,
                request_buffer.as_ptr() as *const core::ffi::c_void,
                response_buffer.len() as u8,
                response_buffer.as_mut_ptr() as *mut core::ffi::c_void,
            )
        };

        if result {
            Ok(postcard::from_bytes::<Response>(&response_buffer)
                .context("unable to parse response")?)
        } else {
            Err(anyhow::anyhow!("unable to communicate with secondary"))
        }
    }

    pub fn secondary_recv<
        'a,
        Request: Deserialize<'a>,
        Response: Default + Serialize + DeserializeOwned,
        F,
    >(
        in_len: u8,
        in_data: *const core::ffi::c_void,
        out_len: u8,
        out_data: *mut core::ffi::c_void,
        f: F,
    ) where
        F: Fn(Request) -> Response,
    {
        let in_data_ptr = in_data as *const u8;
        let incoming_slice = unsafe { alloc::slice::from_raw_parts(in_data_ptr, in_len as usize) };

        let response = if let Ok(incoming) = postcard::from_bytes::<Request>(incoming_slice) {
            f(incoming)
        } else {
            Response::default()
        };

        let out_data_ptr = out_data as *mut u8;
        let outgoing = unsafe { alloc::slice::from_raw_parts_mut(out_data_ptr, out_len as usize) };
        let _ = postcard::to_slice(&response, outgoing);
    }
}

impl Channel {
    pub fn index(&self) -> usize {
        match self {
            Channel::A => 0,
            Channel::Sync => 1,
            Channel::C => 2,
            Channel::D => 3,
            Channel::E => 4,
            Channel::F => 5,
        }
    }

    pub fn to_qmk_id(self) -> i8 {
        match self {
            Channel::A => qmk_sys::serial_transaction_id::USER_CHANNEL_A as i8,
            Channel::Sync => qmk_sys::serial_transaction_id::USER_CHANNEL_B as i8,
            Channel::C => qmk_sys::serial_transaction_id::USER_CHANNEL_C as i8,
            Channel::D => qmk_sys::serial_transaction_id::USER_CHANNEL_D as i8,
            Channel::E => qmk_sys::serial_transaction_id::USER_CHANNEL_E as i8,
            Channel::F => qmk_sys::serial_transaction_id::USER_CHANNEL_F as i8,
        }
    }
}

pub type Bridge = Box<dyn Fn(u8, *const core::ffi::c_void, u8, *mut core::ffi::c_void)>;

static mut SAM_PORTER_BRIDGES: Option<Vec<Option<Bridge>>> = None;

pub fn bridges() -> &'static mut Vec<Option<Bridge>> {
    unsafe {
        #[allow(static_mut_refs)]
        let existing = SAM_PORTER_BRIDGES.as_mut();

        match existing {
            Some(bridges) => bridges,
            None => {
                SAM_PORTER_BRIDGES = Some(vec![None, None, None, None, None, None]);

                #[allow(static_mut_refs)]
                SAM_PORTER_BRIDGES.as_mut().unwrap()
            }
        }
    }
}

macro_rules! bridge_for {
    ($type_name:ident => $type:expr) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $type_name(
            in_len: u8,
            in_data: *const core::ffi::c_void,
            out_len: u8,
            out_data: *mut core::ffi::c_void,
        ) {
            let bridges = bridges();

            if let Some(Some(f)) = bridges.get_mut($type.index()) {
                f(in_len, in_data, out_len, out_data);
            }
        }
    };
}

bridge_for!(bridge_a => Channel::A);
bridge_for!(bridge_sync => Channel::Sync);
bridge_for!(bridge_c => Channel::C);
bridge_for!(bridge_d => Channel::D);
bridge_for!(bridge_e => Channel::E);
bridge_for!(bridge_f => Channel::F);

pub fn listen<
    'a,
    Request: Deserialize<'a> + 'static,
    Response: Default + Serialize + DeserializeOwned + 'static,
    F,
>(
    channel: Channel,
    f: F,
) where
    F: Fn(Request) -> Response + 'static,
{
    let inner_f = Rc::new(Box::new(f));

    let outer_f: Bridge = Box::new(
        move |in_len: u8,
              in_data: *const core::ffi::c_void,
              out_len: u8,
              out_data: *mut core::ffi::c_void| {
            Keyboard::secondary_recv(in_len, in_data, out_len, out_data, inner_f.as_ref());
        },
    );

    let bridges = bridges();
    bridges[channel.index()].replace(outer_f);

    unsafe {
        qmk_sys::transaction_register_rpc(
            channel.to_qmk_id(),
            Some(match channel {
                Channel::A => bridge_a,
                Channel::Sync => bridge_sync,
                Channel::C => bridge_c,
                Channel::D => bridge_d,
                Channel::E => bridge_e,
                Channel::F => bridge_f,
            }),
        );
    }
}
