use alloc::format;

use crate::utils::debug::debug_log;

static mut EEPROM_VALUE: Option<u32> = None;

//

#[allow(clippy::upper_case_acronyms)]
pub struct EEPROM;

impl EEPROM {
    fn read() -> u32 {
        #[allow(static_mut_refs)]
        unsafe {
            match EEPROM_VALUE {
                Some(value) => value,
                None => {
                    let value = qmk_sys::eeconfig_read_user();
                    EEPROM_VALUE = Some(value);
                    debug_log(&format!("[eeprom] load = {:X} = {:#034b}", value, value));
                    value
                }
            }
        }
    }

    fn write() {
        #[allow(static_mut_refs)]
        unsafe {
            if let Some(value) = EEPROM_VALUE {
                qmk_sys::eeconfig_update_user(value);
                debug_log(&format!("[eeprom] write = {:X} = {:#034b}", value, value));
            }
        }
    }

    pub fn get_backlight() -> u8 {
        let value = (Self::read() as u8) & 0xF;
        debug_log(&format!("[eeprom] get_backlight({})", value));
        value
    }

    pub fn set_backlight(value: u8) {
        let existing = Self::read();
        unsafe {
            EEPROM_VALUE = Some((existing & 0xFFFFFFF0) | ((value as u32) & 0xF));
        }
        debug_log(&format!("[eeprom] set_backlight({})", value));
        Self::write();
    }
}
