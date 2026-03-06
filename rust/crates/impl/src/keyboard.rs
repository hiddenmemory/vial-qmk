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

pub struct Keyboard;
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

    pub fn seondary_send<Request: Serialize, Response: Default + Serialize + DeserializeOwned>(
        transaction_id: u32,
        request: Request,
    ) -> Result<Response> {
        let request_buffer =
            postcard::to_allocvec(&request).context("unable to serialize request")?;

        let response_template = Response::default();
        let mut response_buffer = postcard::to_allocvec(&response_template)
            .context("unable to serialize response template")?;

        let result = unsafe {
            qmk_sys::transaction_rpc_exec(
                transaction_id as i8,
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
            Err(anyhow::anyhow!("unable to communicate with seconday"))
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
        mut f: F,
    ) where
        F: FnMut(Request) -> Response,
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
