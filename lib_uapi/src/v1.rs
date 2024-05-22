mod ffi {
    #![allow(unused)]
    use bitflags::bitflags;

    use crate::common::ffi::{CString, Padding, GPIO_MAX_NAME_SIZE};

    const GPIOHANDLES_MAX: usize = 64;

    bitflags! {
        #[derive(Debug, Clone, Copy)]
        pub(crate) struct GpioLineFlags: u32 {
            const KERNEL        = 1 << 0;
            const IS_OUT        = 1 << 1;
            const ACTIVE_LOW    = 1 << 2;
            const OPEN_DRAIN    = 1 << 3;
            const OPEN_SOURCE   = 1 << 4;
            const BIAS_PULL_UP  = 1 << 5;
            const BIAS_PULL_DOWN= 1 << 6;
            const BIAS_DISABLE  = 1 << 7;
        }
    }

    #[repr(C)]
    #[derive(Debug)]
    pub(crate) struct GpioLineInfo {
        pub(crate) line_offset: u32,
        pub(crate) flags: u32,
        pub(crate) name: CString<GPIO_MAX_NAME_SIZE>,
        pub(crate) consumer: CString<GPIO_MAX_NAME_SIZE>,
    }

    #[repr(u32)]
    #[derive(Debug)]
    pub(crate) enum GpioLineChangedType {
        Requested = 1,
        Released = 2,
        Config = 3,
    }

    #[repr(C)]
    #[derive(Debug)]
    pub(crate) struct GpioLineInfoChanged {
        pub(crate) info: GpioLineInfo,
        pub(crate) timestamp: u64,
        pub(crate) event_type: u32,
        pub(crate) padding: Padding<u32, 5>,
    }

    bitflags! {
        #[derive(Debug, Clone, Copy)]
        pub struct GpioHandleFlags: u32 {
            const REQUEST_INPUT          = 1 << 0;
            const REQUEST_OUTPUT         = 1 << 1;
            const REQUEST_ACTIVE_LOW     = 1 << 2;
            const REQUEST_OPEN_DRAIN     = 1 << 3;
            const REQUEST_OPEN_SOURCE    = 1 << 4;
            const REQUEST_BIAS_PULL_UP   = 1 << 5;
            const REQUEST_BIAS_PULL_DOWN = 1 << 6;
            const REQUEST_BIAS_DISABLE   = 1 << 7;
        }
    }

    #[repr(C)]
    #[derive(Debug)]
    pub(crate) struct GpioHandleRequest {
        pub(crate) lineoffsets: [u32; GPIOHANDLES_MAX],
        pub(crate) flags: u32,
        pub(crate) default_values: [u8; GPIOHANDLES_MAX],
        pub(crate) consumer_label: CString<GPIO_MAX_NAME_SIZE>,
        pub(crate) lines: u32,
        pub(crate) fd: libc::c_int,
    }

    #[repr(C)]
    #[derive(Debug)]
    pub(crate) struct GpioHandleConfig {
        pub(crate) flags: u32,
        pub(crate) default_values: [u8; GPIOHANDLES_MAX],
        pub(crate) padding: Padding<u32, 4>,
    }

    #[repr(C)]
    #[derive(Debug)]
    pub(crate) struct GpioHandleData {
        pub(crate) values: [u8; GPIOHANDLES_MAX],
    }

    bitflags! {
        #[derive(Debug, Clone, Copy)]
        pub(crate) struct GpioEventFlags: u32 {
            const REQUEST_RISING_EDGE  = 1 << 0;
            const REQUEST_FALLING_EDGE = 1 << 1;
            const REQUEST_BOTH_EDGES   = Self::REQUEST_RISING_EDGE.bits() | Self::REQUEST_FALLING_EDGE.bits();
        }
    }

    #[repr(C)]
    #[derive(Debug)]
    pub(crate) struct GpioEventRequest {
        pub(crate) lineoffset: u32,
        pub(crate) handleflags: u32,
        pub(crate) eventflags: u32,
        pub(crate) consumer_label: CString<GPIO_MAX_NAME_SIZE>,
        pub(crate) fd: libc::c_int,
    }

    bitflags! {
        #[derive(Debug, Copy, Clone)]
        pub(crate) struct GpioEventType: u32 {
            const RISING_EDGE  = 0x01;
            const FALLING_EDGE = 0x02;
        }
    }

    #[repr(C)]
    #[derive(Debug)]
    pub(crate) struct GpioEventData {
        pub(crate) timestamp: u64,
        pub(crate) id: u32,
    }

    crate::macros::wrap_ioctl!(
        ioctl_readwrite!(
            gpio_get_lineinfo_ioctl,
            crate::common::ffi::GPIO_IOC_MAGIC,
            0x02,
            crate::v1::ffi::GpioLineInfo
        ),
        crate::error::IoctlKind::GetLineInfo
    );

    crate::macros::wrap_ioctl!(
        ioctl_readwrite!(
            gpio_get_linehandle_ioctl,
            crate::common::ffi::GPIO_IOC_MAGIC,
            0x03,
            crate::v1::ffi::GpioHandleRequest
        ),
        crate::error::IoctlKind::GetLine
    );

    crate::macros::wrap_ioctl!(
        ioctl_readwrite!(
            gpio_get_lineevent_ioctl,
            crate::common::ffi::GPIO_IOC_MAGIC,
            0x04,
            crate::v1::ffi::GpioEventRequest
        ),
        crate::error::IoctlKind::GetLineEvent
    );

    crate::macros::wrap_ioctl!(
        ioctl_readwrite!(
            gpiohandle_get_line_values_ioctl,
            crate::common::ffi::GPIO_IOC_MAGIC,
            0x08,
            crate::v1::ffi::GpioHandleData
        ),
        crate::error::IoctlKind::GetValues
    );

    crate::macros::wrap_ioctl!(
        ioctl_readwrite!(
            gpiohandle_set_line_values_ioctl,
            crate::common::ffi::GPIO_IOC_MAGIC,
            0x09,
            crate::v1::ffi::GpioHandleData
        ),
        crate::error::IoctlKind::SetValues
    );

    crate::macros::wrap_ioctl!(
        ioctl_readwrite!(
            gpiohandle_set_config_ioctl,
            crate::common::ffi::GPIO_IOC_MAGIC,
            0x0A,
            crate::v1::ffi::GpioHandleConfig
        ),
        crate::error::IoctlKind::SetLineConfig
    );

    crate::macros::wrap_ioctl!(
        ioctl_readwrite!(
            gpio_get_lineinfo_watch_ioctl,
            crate::common::ffi::GPIO_IOC_MAGIC,
            0x0B,
            crate::v1::ffi::GpioLineInfo
        ),
        crate::error::IoctlKind::GetLineInfo
    );
}
