pub(crate) const GPIO_MAX_NAME_SIZE: usize = 32;
pub(crate) const GPIO_IOC_MAGIC: u8 = 0xB4;

#[derive(Debug)]
#[repr(transparent)]
pub(crate) struct Padding<T, const N: usize>([T; N]);

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
