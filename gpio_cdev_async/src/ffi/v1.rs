use bitflags::bitflags;

use crate::ffi::common::{CString, Padding, GPIO_MAX_NAME_SIZE};

pub(crate) const GPIOHANDLES_MAX: usize = 64;

bitflags! {
    /// Gpio Line Info Flags returned by the kernel.
    ///
    /// Mapping of the flags can be found in the kernel source code:
    /// [gpio.h](https://elixir.bootlin.com/linux/v6.9.2/source/include/uapi/linux/gpio.h#L313)
    #[derive(Debug, Clone, Copy)]
    pub struct GpioLineFlag: u32 {
        const KERNEL         = 1 << 0;
        const IS_OUT         = 1 << 1;
        const ACTIVE_LOW     = 1 << 2;
        const OPEN_DRAIN     = 1 << 3;
        const OPEN_SOURCE    = 1 << 4;
        const BIAS_PULL_UP   = 1 << 5;
        const BIAS_PULL_DOWN = 1 << 6;
        const BIAS_DISABLE   = 1 << 7;
    }
}

/// Infomation about a certain GPIO line.
#[repr(C)]
#[derive(Debug)]
pub(crate) struct GpioLineInfo {
    /// the local offset on this GPIO devices, fill this in
    /// when requesting the line infomation from the kernel.
    pub(crate) line_offset: u32,
    /// the [`GpioLineFlag`] of the GPIO line.
    pub(crate) flags: u32,
    /// the name of the GPIO line.
    pub(crate) name: CString<GPIO_MAX_NAME_SIZE>,
    /// a functional name for the consumer of this GPIO line
    /// as set by whatever is using it.
    pub(crate) consumer: CString<GPIO_MAX_NAME_SIZE>,
}

/// Possible line status change events.
#[repr(u32)]
#[derive(Debug)]
pub enum GpioLineChangedType {
    Requested = 1,
    Released = 2,
    Config = 3,
}

/// Infomation about a change in status of a GPIO line.
#[repr(C)]
#[derive(Debug)]
pub(crate) struct GpioLineInfoChanged {
    /// update line infomation.
    pub(crate) info: GpioLineInfo,
    /// estimate of time of status change occurrence, in nanoseconds.
    pub(crate) timestamp: u64,
    /// the [`GpioLineChangedType`] of the event.
    pub(crate) event_type: u32,
    /// reserved for future use.
    pub(crate) padding: Padding<u32, 5>,
}

bitflags! {
    /// Line Request Flags.
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

/// Information about a GPIO handle request.
#[repr(C)]
#[derive(Debug)]
pub(crate) struct GpioHandleRequest {
    /// an array of desired GPIO line offsets, specified
    /// by offset index for the associated GPIO device.
    pub(crate) lineoffsets: [u32; GPIOHANDLES_MAX],
    /// desired [`GpioHandleFlags`] for the GPIO handle.
    pub(crate) flags: u32,
    /// if `REQUEST_OUTPUT` is set for a requested line,
    /// this specifies the default output value, should be 0 (inactive) or 1 (active).
    pub(crate) default_values: [u8; GPIOHANDLES_MAX],
    /// a desired consumer label for the GPIO line(s).
    pub(crate) consumer_label: CString<GPIO_MAX_NAME_SIZE>,
    /// number of lines requested.
    pub(crate) lines: u32,
    /// after a successful request, this is the file
    /// descriptor for the requested GPIO handle.
    pub(crate) fd: libc::c_int,
}

/// Configuration for a GPIO handle request.
#[repr(C)]
#[derive(Debug)]
pub(crate) struct GpioHandleConfig {
    /// the desired [`GpioHandleFlags`] for the GPIO handle.
    pub(crate) flags: u32,
    /// if `REQUEST_OUTPUT` is set for a requested line,
    /// this specifies the default output value, should
    /// be 0 (inactive) or 1 (active).
    pub(crate) default_values: [u8; GPIOHANDLES_MAX],
    pub(crate) padding: Padding<u32, 4>,
}

/// Information of values on a GPIO handle
#[repr(C)]
#[derive(Debug)]
pub(crate) struct GpioHandleData {
    /// when getting the state of lines this contains the
    /// current state of a line, when setting the state of
    /// lines these should contain desired target state.
    /// States are 0 (inactive) or 1 (active).
    pub(crate) values: [u8; GPIOHANDLES_MAX],
}

bitflags! {
    /// Event Request flags
    #[derive(Debug, Clone, Copy)]
    pub(crate) struct GpioEventFlags: u32 {
        const REQUEST_RISING_EDGE  = 1 << 0;
        const REQUEST_FALLING_EDGE = 1 << 1;
        const REQUEST_BOTH_EDGES   = Self::REQUEST_RISING_EDGE.bits() | Self::REQUEST_FALLING_EDGE.bits();
    }
}

/// Information about a GPIO event request.
#[repr(C)]
#[derive(Debug)]
pub(crate) struct GpioEventRequest {
    /// the desired line to subscribe to events from,
    /// specified by offset index for the associated GPIO device.
    pub(crate) lineoffset: u32,
    /// desired [`GpioHandleFlags`] flags for the desired GPIO line
    pub(crate) handleflags: u32,
    /// desired [`GpioEventFlags`] for the desired GPIO event line
    pub(crate) eventflags: u32,
    pub(crate) consumer_label: CString<GPIO_MAX_NAME_SIZE>,
    pub(crate) fd: libc::c_int,
}

bitflags! {
    /// GPIO Event Types
    #[derive(Debug, Copy, Clone)]
    pub(crate) struct GpioEventType: u32 {
        const RISING_EDGE  = 0x01;
        const FALLING_EDGE = 0x02;
    }
}

bitflags! {
    /// Event Request flags
    #[derive(Debug, Clone, Copy)]
    pub(crate) struct GpioEventRequestFlags: u32 {
        const REQUEST_RISING_EDGE  = 1 << 0;
        const REQUEST_FALLING_EDGE = 1 << 1;
        const REQUEST_BOTH_EDGES   = Self::REQUEST_RISING_EDGE.bits() | Self::REQUEST_FALLING_EDGE.bits();
    }
}

/// The actual event being pushed to userspace
#[repr(C)]
#[derive(Debug)]
pub(crate) struct GpioEventData {
    pub(crate) timestamp: u64,
    /// event identifier, one of [`GpioEventType`]
    pub(crate) id: u32,
}

crate::macros::wrap_ioctl!(
    ioctl_readwrite!(
        gpio_get_lineinfo_ioctl,
        crate::ffi::common::GPIO_IOC_MAGIC,
        0x02,
        crate::ffi::v1::GpioLineInfo
    ),
    crate::error::IoctlKind::GetLineInfo
);

crate::macros::wrap_ioctl!(
    ioctl_readwrite!(
        gpio_get_linehandle_ioctl,
        crate::ffi::common::GPIO_IOC_MAGIC,
        0x03,
        crate::ffi::v1::GpioHandleRequest
    ),
    crate::error::IoctlKind::GetLine
);

crate::macros::wrap_ioctl!(
    ioctl_readwrite!(
        gpio_get_lineevent_ioctl,
        crate::ffi::common::GPIO_IOC_MAGIC,
        0x04,
        crate::ffi::v1::GpioEventRequest
    ),
    crate::error::IoctlKind::GetLineEvent
);

crate::macros::wrap_ioctl!(
    ioctl_readwrite!(
        gpiohandle_get_line_values_ioctl,
        crate::ffi::common::GPIO_IOC_MAGIC,
        0x08,
        crate::ffi::v1::GpioHandleData
    ),
    crate::error::IoctlKind::GetValues
);

crate::macros::wrap_ioctl!(
    ioctl_readwrite!(
        gpiohandle_set_line_values_ioctl,
        crate::ffi::common::GPIO_IOC_MAGIC,
        0x09,
        crate::ffi::v1::GpioHandleData
    ),
    crate::error::IoctlKind::SetValues
);

crate::macros::wrap_ioctl!(
    ioctl_readwrite!(
        gpiohandle_set_config_ioctl,
        crate::ffi::common::GPIO_IOC_MAGIC,
        0x0A,
        crate::ffi::v1::GpioHandleConfig
    ),
    crate::error::IoctlKind::SetLineConfig
);

crate::macros::wrap_ioctl!(
    ioctl_readwrite!(
        gpio_get_lineinfo_watch_ioctl,
        crate::ffi::common::GPIO_IOC_MAGIC,
        0x0B,
        crate::ffi::v1::GpioLineInfo
    ),
    crate::error::IoctlKind::GetLineInfo
);

mod helper {
    use super::*;

    impl From<u32> for GpioLineChangedType {
        fn from(value: u32) -> Self {
            match value {
                1 => Self::Requested,
                2 => Self::Released,
                _ => Self::Config,
            }
        }
    }
}
