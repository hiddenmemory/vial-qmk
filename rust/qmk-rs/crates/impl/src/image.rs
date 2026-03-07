use crate::{
    display::Display,
    utils::{Point, Size, Sizeable},
};

#[derive(Copy, Clone)]
pub struct Image {
    pub id: usize,
    pub size: Size,
    pub frames: u16,
    pub handle: qmk_sys::painter_image_handle_t,
}

impl core::fmt::Debug for Image {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Image")
            .field("id", &self.id)
            .field("size", &self.size)
            .field("frames", &self.frames)
            .field("handle", &self.handle)
            .finish()
    }
}

impl Image {
    pub fn new(source: &[u8]) -> Image {
        unsafe {
            let handle = qmk_sys::qp_load_image_mem(source.as_ptr() as *const core::ffi::c_void);
            let id = source.as_ptr() as usize;
            let width = (*handle).width;
            let height = (*handle).height;
            let frames = (*handle).frame_count;
            Image {
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

impl Sizeable for Image {
    fn size(&self) -> Size {
        self.size
    }
}

impl Sizeable for &Image {
    fn size(&self) -> Size {
        self.size
    }
}
