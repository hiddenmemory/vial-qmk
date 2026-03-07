#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd)]
pub enum HostOS {
    #[default]
    Unknown,
    Unsure,
    Linux,
    Windows,
    MacOS,
    Ios,
}

impl HostOS {
    pub fn current() -> HostOS {
        let detected = unsafe { qmk_sys::detected_host_os() };

        if detected == qmk_sys::os_variant_t::OS_MACOS {
            HostOS::MacOS
        } else if detected == qmk_sys::os_variant_t::OS_LINUX {
            HostOS::Linux
        } else if detected == qmk_sys::os_variant_t::OS_WINDOWS {
            HostOS::Windows
        } else if detected == qmk_sys::os_variant_t::OS_IOS {
            HostOS::Ios
        } else if detected == qmk_sys::os_variant_t::OS_UNSURE {
            HostOS::Unsure
        } else {
            HostOS::Unknown
        }
    }

    pub fn name(&self) -> &str {
        match self {
            HostOS::Unknown => "?",
            HostOS::Unsure => "Unsure",
            HostOS::Linux => "Linux",
            HostOS::Windows => "Windows",
            HostOS::MacOS => "macOS",
            HostOS::Ios => "iOS",
        }
    }
}
