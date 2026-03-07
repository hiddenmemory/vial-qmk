#[derive(Copy, Clone)]
pub struct Font {
    pub line_height: u16,
    pub handle: qmk_sys::painter_font_handle_t,
}

impl Font {
    pub fn new(source: &[u8]) -> Font {
        unsafe {
            let handle = qmk_sys::qp_load_font_mem(source.as_ptr() as *const core::ffi::c_void);
            let line_height = (*handle).line_height as u16;
            Font {
                line_height,
                handle,
            }
        }
    }
}
