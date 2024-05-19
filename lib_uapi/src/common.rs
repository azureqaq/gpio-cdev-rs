pub(crate) mod helper {
    use std::{ffi::CString, fmt::Display};

    use super::ffi;

    impl<const N: usize> Display for ffi::CString<N> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", cstr_to_string(self.0))
        }
    }

    fn cstr_to_string(cstr: impl AsRef<[libc::c_char]>) -> String {
        String::from_utf8_lossy(cstr.as_ref()).to_string()
    }
}

pub(crate) mod ffi {
    pub(crate) const GPIO_MAX_NAME_SIZE: usize = 32;
    pub(crate) const GPIO_IOC_MAGIC: u8 = 0xB4;

    #[derive(Debug)]
    #[repr(transparent)]
    pub(crate) struct Padding<T, const N: usize>([T; N]);

    #[derive(Debug)]
    #[repr(transparent)]
    pub(crate) struct CString<const N: usize>(pub(crate) [libc::c_char; N]);

    /// Information about a certain GPIO chip
    #[repr(C)]
    pub(crate) struct GpioChipInfo {
        pub(crate) name: CString<GPIO_MAX_NAME_SIZE>,
        pub(crate) lable: CString<GPIO_MAX_NAME_SIZE>,
        /// number of GPIO lines on this chip
        pub(crate) lines: u32,
    }

    crate::macros::wrap_ioctl!(
        ioctl_read!(
            gpio_get_chipinfo_ioctl,
            crate::common::ffi::GPIO_IOC_MAGIC,
            0x01,
            crate::common::ffi::GpioChipInfo
        ),
        crate::error::IoctlKind::GetChipInfo
    );

    crate::macros::wrap_ioctl!(
        ioctl_readwrite!(
            gpio_get_lineinfo_unwatch_ioctl,
            crate::common::ffi::GPIO_IOC_MAGIC,
            0x0C,
            u32
        ),
        crate::error::IoctlKind::GetLineInfo
    );
}
