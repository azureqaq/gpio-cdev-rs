use std::{
    borrow::Cow,
    ffi::CStr,
    fmt::Debug,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
};

use crate::error::Result;
pub use ffi::GpioV2LineFlag as LineFlag;

pub struct LineRequest {
    inner: ffi::GpioV2LineRequest,
}

impl LineRequest {
    pub fn num_lines(&self) -> u32 {
        self.inner.num_lines
    }

    pub fn offsets(&self) -> &[u32] {
        debug_assert!(self.num_lines() > 0);
        self.inner
            .offsets
            .get(..self.inner.num_lines as usize)
            .unwrap_or_default()
    }

    pub fn consumer(&self) -> Cow<'_, str> {
        CStr::from_bytes_until_nul(self.inner.consumer.0.as_slice())
            .unwrap_or_default()
            .to_string_lossy()
    }

    pub fn event_buffer_size(&self) -> u32 {
        self.inner.event_buffer_size
    }

    pub fn fd(&self) -> libc::c_int {
        self.inner.fd
    }

    pub fn config(&self) -> &LineConfig {
        let c = &self.inner.config;
        unsafe { &*(c as *const ffi::GpioV2LineConfig as *const LineConfig) }
    }
}

impl Debug for LineRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LineRequest")
            .field("offsets", &self.offsets())
            .field("num_lines", &self.num_lines())
            .field("consumer", &self.consumer())
            .field("config", &self.config())
            .finish()
    }
}

#[repr(transparent)]
pub struct LineConfig {
    inner: ffi::GpioV2LineConfig,
}

impl LineConfig {
    pub fn flags(&self) -> LineFlag {
        LineFlag::from_bits_retain(self.inner.flags)
    }

    pub fn num_attrs(&self) -> u32 {
        self.inner.num_attrs
    }

    pub fn attrs(&self) -> &[LineConfigAttribute] {
        let ptr = self
            .inner
            .attrs
            .get(..self.num_attrs() as usize)
            .unwrap_or_default();
        debug_assert!(ptr.len() <= isize::MAX as usize);
        unsafe { std::slice::from_raw_parts(ptr.as_ptr() as *const LineConfigAttribute, ptr.len()) }
    }
}

impl Debug for LineConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LineConfig")
            .field("flags", &self.flags())
            .field("num_attrs", &self.num_attrs())
            .field("attrs", &self.attrs())
            .finish()
    }
}

#[repr(transparent)]
pub struct LineConfigAttribute {
    inner: ffi::GpioV2LineConfigAttribute,
}

impl LineConfigAttribute {
    pub fn mask(&self) -> libc::c_ulong {
        self.inner.mask
    }

    pub fn attr(&self) -> &LineAttribute {
        unsafe { &*(&self.inner.attr as *const ffi::GpioV2LineAttribute as *const LineAttribute) }
    }
}

impl Debug for LineConfigAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LineConfigAttribute")
            .field("attr", &self.attr())
            .field("mask", &self.mask())
            .finish()
    }
}

pub struct LineInfo {
    inner: ffi::GpioV2LineInfo,
}

impl LineInfo {
    pub fn name(&self) -> Cow<'_, str> {
        CStr::from_bytes_until_nul(self.inner.name.0.as_slice())
            .unwrap_or_default()
            .to_string_lossy()
    }

    pub fn consumer(&self) -> Cow<'_, str> {
        CStr::from_bytes_until_nul(self.inner.consumer.0.as_slice())
            .unwrap_or_default()
            .to_string_lossy()
    }

    pub fn offset(&self) -> u32 {
        self.inner.offset
    }

    pub fn num_attrs(&self) -> u32 {
        self.inner.num_attrs
    }

    pub fn flags(&self) -> libc::c_ulong {
        self.inner.flags
    }

    pub fn attrs(&self) -> &[LineAttribute] {
        let lst = self
            .inner
            .attrs
            .get(..self.num_attrs() as usize)
            .unwrap_or_default();
        debug_assert!(lst.len() <= isize::MAX as usize);
        unsafe { std::slice::from_raw_parts(lst.as_ptr() as *const LineAttribute, lst.len()) }
    }
}

impl Debug for LineInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LineInfo")
            .field("name", &self.name())
            .field("consumer", &self.consumer())
            .field("offset", &self.offset())
            .field("num_attrs", &self.num_attrs())
            .field("flags", &self.flags())
            .field("attrs", &self.attrs())
            .finish()
    }
}

#[repr(transparent)]
pub struct LineAttribute {
    inner: ffi::GpioV2LineAttribute,
}

impl LineAttribute {
    pub fn get_value(&self) -> LineAttributeValue {
        match ffi::GpioV2LineAttrId::from(self.inner.id) {
            ffi::GpioV2LineAttrId::Flags => {
                LineAttributeValue::Flags(unsafe { self.inner.u.flags })
            }
            ffi::GpioV2LineAttrId::OutputValues => {
                LineAttributeValue::Values(unsafe { self.inner.u.values })
            }
            ffi::GpioV2LineAttrId::Debounce => {
                LineAttributeValue::DebouncePeriodUs(unsafe { self.inner.u.debounce_period_us })
            }
        }
    }
}

impl Debug for LineAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LineAttribute")
            .field(&self.get_value())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineAttributeValue {
    Flags(libc::c_ulong),
    Values(libc::c_ulong),
    DebouncePeriodUs(u32),
}

#[derive(Debug)]
pub struct LineHandle {
    fd: OwnedFd,
    mask: libc::c_ulong,
}

impl LineHandle {
    pub fn get_bites(&self) -> Result<libc::c_ulong> {
        let mut data: ffi::GpioV2LineValues = unsafe { std::mem::zeroed() };
        data.mask = self.mask;
        ffi::gpio_v2_line_get_values_ioctl(self.fd.as_raw_fd(), &mut data)?;
        Ok(data.bits)
    }

    pub fn set_bites(&self, bites: libc::c_ulong) -> Result<()> {
        let mut data: ffi::GpioV2LineValues = unsafe { std::mem::zeroed() };
        data.mask = self.mask;
        ffi::gpio_v2_line_set_values_ioctl(self.fd.as_raw_fd(), &mut data)?;
        Ok(())
    }

    pub fn set_bites_with_submask(&self, mask: libc::c_ulong) -> Result<()> {
        let mut data: ffi::GpioV2LineValues = unsafe { std::mem::zeroed() };
        data.mask = self.mask & mask;
        ffi::gpio_v2_line_set_values_ioctl(self.fd.as_raw_fd(), &mut data)?;
        Ok(())
    }

    pub fn mask(&self) -> libc::c_ulong {
        self.mask
    }
}

#[derive(Debug)]
pub struct LineRequestBuilder {
    inner: ffi::GpioV2LineRequest,
}

impl LineRequestBuilder {
    pub fn new(offsets: &[u32], flags: LineFlag, consumer: impl AsRef<str>) -> Self {
        let offsets_len = offsets.len().min(ffi::GPIO_V2_LINES_MAX);
        let consumer = consumer.as_ref().as_bytes();
        let consumer_len = consumer.len().min(crate::common::ffi::GPIO_MAX_NAME_SIZE);

        let mut inner: ffi::GpioV2LineRequest = unsafe { std::mem::zeroed() };

        inner.config.flags = flags.bits();
        inner
            .offsets
            .get_mut(..offsets_len)
            .unwrap()
            .copy_from_slice(&offsets[..offsets_len]);
        inner.num_lines = offsets_len as u32;
        inner
            .consumer
            .0
            .get_mut(..consumer_len)
            .unwrap()
            .copy_from_slice(&consumer[..consumer_len]);

        Self { inner }
    }

    pub fn build(self) -> Result<LineRequest> {
        // TODO: check config
        Ok(LineRequest { inner: self.inner })
    }
}

pub fn get_line(fd: impl AsRawFd, request: &mut LineRequest) -> Result<LineHandle> {
    ffi::gpio_v2_get_line_ioctl(fd.as_raw_fd(), &mut request.inner)?;
    Ok(LineHandle {
        fd: unsafe { OwnedFd::from_raw_fd(request.fd()) },
        mask: helper::offsets_to_mask(request.offsets()),
    })
}

pub fn get_lineinfo(fd: impl AsRawFd, offset: u32) -> Result<LineInfo> {
    let mut inner: ffi::GpioV2LineInfo = unsafe { std::mem::zeroed() };
    inner.offset = offset;
    ffi::gpio_v2_get_lineinfo_ioctl(fd.as_raw_fd(), &mut inner)?;
    Ok(LineInfo { inner })
}

mod helper {
    use super::ffi;

    impl From<u32> for ffi::GpioV2LineAttrId {
        fn from(value: u32) -> Self {
            debug_assert!(matches!(value, 1..=3));
            match value {
                1 => Self::Flags,
                2 => Self::OutputValues,
                _ => Self::Debounce,
            }
        }
    }

    pub(crate) fn offsets_to_mask(offsets: &[u32]) -> libc::c_ulong {
        let mut res: libc::c_ulong = 0;
        for &n in offsets {
            res |= (1 << n);
        }
        res
    }
}

mod ffi {
    use std::fmt::Debug;

    use bitflags::bitflags;

    use crate::common::ffi::{CString, Padding, GPIO_MAX_NAME_SIZE};

    pub(crate) const GPIO_V2_LINES_MAX: usize = 64;
    const GPIO_V2_LINE_NUM_ATTRS_MAX: usize = 10;

    bitflags! {
        #[derive(Debug, Clone, Copy)]
        pub struct GpioV2LineFlag: libc::c_ulong {
            const GPIO_V2_LINE_FLAG_USED                 = 1 << 0;
            const GPIO_V2_LINE_FLAG_ACTIVE_LOW           = 1 << 1;
            const GPIO_V2_LINE_FLAG_INPUT                = 1 << 2;
            const GPIO_V2_LINE_FLAG_OUTPUT               = 1 << 3;
            const GPIO_V2_LINE_FLAG_EDGE_RISING          = 1 << 4;
            const GPIO_V2_LINE_FLAG_EDGE_FALLING         = 1 << 5;
            const GPIO_V2_LINE_FLAG_OPEN_DRAIN           = 1 << 6;
            const GPIO_V2_LINE_FLAG_OPEN_SOURCE          = 1 << 7;
            const GPIO_V2_LINE_FLAG_BIAS_PULL_UP         = 1 << 8;
            const GPIO_V2_LINE_FLAG_BIAS_PULL_DOWN       = 1 << 9;
            const GPIO_V2_LINE_FLAG_BIAS_DISABLED        = 1 << 10;
            const GPIO_V2_LINE_FLAG_EVENT_CLOCK_REALTIME = 1 << 11;
            const GPIO_V2_LINE_FLAG_EVENT_CLOCK_HTE      = 1 << 12;
        }
    }

    #[derive(Debug)]
    #[repr(C)]
    pub(crate) struct GpioV2LineValues {
        pub(crate) bits: libc::c_ulong,
        pub(crate) mask: libc::c_ulong,
    }

    #[derive(Debug)]
    #[repr(u32)]
    pub(crate) enum GpioV2LineAttrId {
        Flags = 1,
        OutputValues = 2,
        Debounce = 3,
    }

    #[repr(C)]
    pub(crate) union Union {
        pub(crate) flags: libc::c_ulong,
        pub(crate) values: libc::c_ulong,
        pub(crate) debounce_period_us: u32,
    }

    impl Debug for Union {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            unsafe {
                match self {
                    Self { flags } => write!(f, "{}", flags),
                    Self { values } => write!(f, "{}", values),
                    Self { debounce_period_us } => write!(f, "{}", debounce_period_us),
                }
            }
        }
    }

    #[derive(Debug)]
    #[repr(C)]
    pub(crate) struct GpioV2LineAttribute {
        pub(crate) id: u32,
        pub(crate) padding: Padding<u32, 1>,
        pub(crate) u: Union,
    }

    #[derive(Debug)]
    #[repr(C)]
    pub(crate) struct GpioV2LineConfigAttribute {
        pub(crate) attr: GpioV2LineAttribute,
        pub(crate) mask: libc::c_ulong,
    }

    #[derive(Debug)]
    #[repr(C)]
    pub(crate) struct GpioV2LineConfig {
        pub(crate) flags: libc::c_ulong,
        pub(crate) num_attrs: u32,
        pub(crate) padding: Padding<u32, 5>,
        pub(crate) attrs: [GpioV2LineConfigAttribute; GPIO_V2_LINE_NUM_ATTRS_MAX],
    }

    #[derive(Debug)]
    #[repr(C)]
    pub(crate) struct GpioV2LineRequest {
        pub(crate) offsets: [u32; GPIO_V2_LINES_MAX],
        pub(crate) consumer: CString<GPIO_MAX_NAME_SIZE>,
        pub(crate) config: GpioV2LineConfig,
        pub(crate) num_lines: u32,
        pub(crate) event_buffer_size: u32,
        pub(crate) padding: Padding<u32, 5>,
        pub(crate) fd: libc::c_int,
    }

    #[derive(Debug)]
    #[repr(C)]
    pub(crate) struct GpioV2LineInfo {
        pub(crate) name: CString<GPIO_MAX_NAME_SIZE>,
        pub(crate) consumer: CString<GPIO_MAX_NAME_SIZE>,
        pub(crate) offset: u32,
        pub(crate) num_attrs: u32,
        pub(crate) flags: libc::c_ulong,
        pub(crate) attrs: [GpioV2LineAttribute; GPIO_V2_LINE_NUM_ATTRS_MAX],
        pub(crate) padding: Padding<u32, 4>,
    }

    #[derive(Debug)]
    #[repr(u32)]
    pub(crate) enum GpioV2LineChangedType {
        Requested = 1,
        Released = 2,
        Config = 3,
    }

    #[derive(Debug)]
    #[repr(C)]
    pub(crate) struct GpioV2LineInfoChanged {
        pub(crate) info: GpioV2LineInfo,
        pub(crate) timestamp_ns: libc::c_ulong,
        pub(crate) event_type: u32,
        pub(crate) padding: Padding<u32, 5>,
    }

    #[derive(Debug)]
    #[repr(u32)]
    pub(crate) enum GpioV2LineEventId {
        RisingEdge = 1,
        FallingEdge = 2,
    }

    #[derive(Debug)]
    #[repr(C)]
    pub(crate) struct GpioV2LineEvent {
        pub(crate) timestamp_ns: libc::c_ulong,
        pub(crate) id: u32,
        pub(crate) offset: u32,
        pub(crate) seqno: u32,
        pub(crate) line_seqno: u32,
        pub(crate) padding: Padding<u32, 6>,
    }

    crate::macros::wrap_ioctl!(
        ioctl_readwrite!(
            gpio_v2_get_lineinfo_ioctl,
            crate::common::ffi::GPIO_IOC_MAGIC,
            0x05,
            crate::v2::ffi::GpioV2LineInfo
        ),
        crate::error::IoctlKind::GetLineInfo
    );

    crate::macros::wrap_ioctl!(
        ioctl_readwrite!(
            gpio_v2_get_lineinfo_watch_ioctl,
            crate::common::ffi::GPIO_IOC_MAGIC,
            0x06,
            crate::v2::ffi::GpioV2LineInfo
        ),
        crate::error::IoctlKind::GetLineInfo
    );

    crate::macros::wrap_ioctl!(
        ioctl_readwrite!(
            gpio_v2_get_line_ioctl,
            crate::common::ffi::GPIO_IOC_MAGIC,
            0x07,
            crate::v2::ffi::GpioV2LineRequest
        ),
        crate::error::IoctlKind::GetLine
    );

    crate::macros::wrap_ioctl!(
        ioctl_readwrite!(
            gpio_v2_line_set_config_ioctl,
            crate::common::ffi::GPIO_IOC_MAGIC,
            0x0D,
            crate::v2::ffi::GpioV2LineConfig
        ),
        crate::error::IoctlKind::SetLineConfig
    );

    crate::macros::wrap_ioctl!(
        ioctl_readwrite!(
            gpio_v2_line_get_values_ioctl,
            crate::common::ffi::GPIO_IOC_MAGIC,
            0x0E,
            crate::v2::ffi::GpioV2LineValues
        ),
        crate::error::IoctlKind::GetValues
    );

    crate::macros::wrap_ioctl!(
        ioctl_readwrite!(
            gpio_v2_line_set_values_ioctl,
            crate::common::ffi::GPIO_IOC_MAGIC,
            0x0F,
            crate::v2::ffi::GpioV2LineValues
        ),
        crate::error::IoctlKind::SetValues
    );
}
