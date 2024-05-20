use std::{borrow::Cow, ffi::CStr, fmt::Display};

use self::ffi::{GpioV2LineConfig, GpioV2LineRequest};

pub struct LineRequest {
    inner: GpioV2LineRequest,
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
        debug_assert!(self.inner.fd > 0);
        self.inner.fd
    }
}

impl Display for LineRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LineRequest")
            .field("offsets", &self.offsets())
            .field("num_lines", &self.num_lines())
            .field("consumer", &self.consumer())
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
        CStr::from_bytes_until_nul(self.inner.name.0.as_slice())
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
}

mod ffi {
    use std::fmt::Debug;

    use bitflags::bitflags;

    use crate::common::ffi::{CString, Padding, GPIO_MAX_NAME_SIZE};

    const GPIO_V2_LINES_MAX: usize = 64;
    const GPIO_V2_LINE_NUM_ATTRS_MAX: usize = 10;

    bitflags! {
        #[derive(Debug, Clone, Copy)]
        pub(crate) struct GpioV2LineFlag: libc::c_ulong {
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
        flags: libc::c_ulong,
        values: libc::c_ulong,
        debounce_period_us: u32,
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
