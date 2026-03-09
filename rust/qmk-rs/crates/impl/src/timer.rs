use crate::keyboard::Keyboard;

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
    #[inline(always)]
    pub fn idle_time_remaining() -> u32 {
        qmk_sys::QUANTUM_PAINTER_DISPLAY_TIMEOUT.saturating_sub(Keyboard::last_activity_elapsed())
    }
}
