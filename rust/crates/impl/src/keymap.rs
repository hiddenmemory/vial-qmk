pub struct KeyMap;

impl KeyMap {
    #[inline]
    pub fn get_layer() -> u8 {
        unsafe { qmk_sys::biton(qmk_sys::layer_state) }
    }

    #[inline]
    pub fn layer_count() -> u8 {
        unsafe { qmk_sys::keymap_layer_count() }
    }
}
