use crate::{
    display::Display,
    utils::{Point, Size, Sizeable},
};

#[derive(Copy, Clone)]
pub struct QmkImage {
    pub id: usize,
    pub size: Size,
    pub frames: u16,
    pub handle: qmk_sys::painter_image_handle_t,
}

unsafe impl Sync for QmkImage {}
unsafe impl Send for QmkImage {}

impl core::fmt::Debug for QmkImage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Image")
            .field("id", &self.id)
            .field("size", &self.size)
            .field("frames", &self.frames)
            .field("handle", &self.handle)
            .finish()
    }
}

#[allow(dead_code)]
impl QmkImage {
    pub fn new(source: &[u8]) -> QmkImage {
        unsafe {
            let handle = qmk_sys::qp_load_image_mem(source.as_ptr() as *const core::ffi::c_void);
            let id = source.as_ptr() as usize;
            let width = (*handle).width;
            let height = (*handle).height;
            let frames = (*handle).frame_count;
            QmkImage {
                id,
                size: Size { width, height },
                frames,
                handle,
            }
        }
    }

    pub fn draw(&self, position: Point, display: &Display) {
        unsafe {
            qmk_sys::qp_drawimage(display.device, position.x, position.y, self.handle);
        }
    }
}

impl Sizeable for QmkImage {
    fn size(&self) -> Size {
        self.size
    }
}

impl Sizeable for &QmkImage {
    fn size(&self) -> Size {
        self.size
    }
}
