pub(crate) const GPIO_MAX_NAME_SIZE: usize = 32;
pub(crate) const GPIO_IOC_MAGIC: u8 = 0xB4;

#[derive(Debug)]
#[repr(transparent)]
pub(crate) struct Padding<T, const N: usize>(pub(crate) [T; N]);

#[derive(Debug)]
#[repr(transparent)]
pub(crate) struct CString<const N: usize>(pub(crate) [libc::c_char; N]);

/// Information about a certain GPIO chip
#[derive(Debug)]
#[repr(C)]
pub(crate) struct GpioChipInfo {
    pub(crate) name: CString<GPIO_MAX_NAME_SIZE>,
    pub(crate) label: CString<GPIO_MAX_NAME_SIZE>,
    /// number of GPIO lines on this chip
    pub(crate) lines: u32,
}

crate::macros::wrap_ioctl!(
    ioctl_read!(
        gpio_get_chipinfo_ioctl,
        crate::ffi::common::GPIO_IOC_MAGIC,
        0x01,
        crate::ffi::common::GpioChipInfo
    ),
    crate::error::IoctlKind::GetChipInfo
);

crate::macros::wrap_ioctl!(
    ioctl_readwrite!(
        gpio_get_lineinfo_unwatch_ioctl,
        crate::ffi::common::GPIO_IOC_MAGIC,
        0x0C,
        u32
    ),
    crate::error::IoctlKind::GetLineInfo
);

pub(crate) mod helper {
    use std::{borrow::Cow, ffi::CStr, fmt::Display};

    use super::CString;

    impl<const N: usize> CString<N> {
        pub(crate) fn to_string_lossy(&self) -> Cow<'_, str> {
            CStr::from_bytes_until_nul(self.0.as_slice())
                .unwrap_or_default()
                .to_string_lossy()
        }
    }

    impl<const N: usize> Display for CString<N> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.to_string_lossy())
        }
    }

    impl<const N: usize, T> From<T> for CString<N>
    where
        T: AsRef<str>,
    {
        fn from(value: T) -> Self {
            let value = value.as_ref().as_bytes();
            let len = value.len().min(N);
            let mut buf = [b'\0'; N];
            // SAFETY: `len` is always less than or equal to `N`
            buf[..len].copy_from_slice(&value[..len]);
            Self(buf)
        }
    }
}
