pub struct Timer;

impl Timer {
    #[inline(always)]
    pub fn read() -> u32 {
        unsafe { qmk_sys::timer_read32() }
    }
    #[inline(always)]
    pub fn elapsed(since: u32) -> u32 {
        unsafe { qmk_sys::timer_elapsed32(since) }
    }
}
