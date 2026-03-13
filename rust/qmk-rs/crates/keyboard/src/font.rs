use alloc::ffi::CString;

use crate::utils::Size;

#[derive(Copy, Clone)]
pub struct QmkFont {
    pub line_height: u16,
    pub handle: qmk_sys::painter_font_handle_t,
}

#[allow(dead_code)]
impl QmkFont {
    pub fn new(source: &[u8]) -> QmkFont {
        unsafe {
            let handle = qmk_sys::qp_load_font_mem(source.as_ptr() as *const core::ffi::c_void);
            let line_height = (*handle).line_height as u16;
            QmkFont {
                line_height,
                handle,
            }
        }
    }

    pub fn size_of(&self, text: &CString) -> Size {
        Size {
            width: unsafe { qmk_sys::qp_textwidth(self.handle, text.as_ptr()) as u16 },
            height: self.line_height,
        }
    }
}
