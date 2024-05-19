use std::{ffi::CStr, os::fd::AsRawFd};

use crate::error::Result;

use self::ffi::GpioChipInfo;

pub struct ChipInfo {
    inner: GpioChipInfo,
}

impl ChipInfo {
    pub fn name(&self) -> &CStr {
        CStr::from_bytes_until_nul(self.inner.name.0.as_slice()).unwrap_or_default()
    }

    pub fn label(&self) -> &CStr {
        CStr::from_bytes_until_nul(self.inner.label.0.as_slice()).unwrap_or_default()
    }

    pub fn lines(&self) -> u32 {
        self.inner.lines
    }
}

/// Get the publicly information for a chip
pub fn get_chipinfo(fd: impl AsRawFd) -> Result<ChipInfo> {
    let mut inner = unsafe { std::mem::zeroed() };
    ffi::gpio_get_chipinfo_ioctl(fd.as_raw_fd(), &mut inner)?;
    Ok(ChipInfo { inner })
}

/// Remove the line from the list of lines being watched on this chip.
///
/// # Errors
/// - Unwatching a line that is not watched is an error(`EBUSY`)
pub fn get_lineinfo_unwatch(fd: impl AsRawFd, offset: u32) -> Result<u32> {
    let mut offset = offset;
    ffi::gpio_get_lineinfo_unwatch_ioctl(fd.as_raw_fd(), &mut offset)?;
    Ok(offset)
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
        pub(crate) label: CString<GPIO_MAX_NAME_SIZE>,
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
