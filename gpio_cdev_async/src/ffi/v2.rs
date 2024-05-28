use std::fmt::Debug;

use bitflags::bitflags;

use crate::ffi::common::{CString, Padding, GPIO_MAX_NAME_SIZE};

pub(crate) const GPIO_V2_LINES_MAX: usize = 64;
pub(crate) const GPIO_V2_LINE_NUM_ATTRS_MAX: usize = 10;

bitflags! {
    /// [`GpioV2LineAttribute`] flags
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

/// Values of GPIO lines
#[derive(Debug)]
#[repr(C)]
pub(crate) struct GpioV2LineValues {
    /// a bitmap containing the value of the lines, set to 1 for active and 0 for inactive.
    pub(crate) bits: libc::c_ulong,
    /// a bitmap identifying the lines to get or set, with each bit number
    /// corresponding to the index into the `GpioV2LineRequest.offsets` array.
    pub(crate) mask: libc::c_ulong,
}

/// [`GpioV2LineAttribute`] id
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

/// A configurable attribute of a line
#[repr(C)]
pub(crate) struct GpioV2LineAttribute {
    /// attribute identifier with value from [`GpioV2LineAttrId`]
    pub(crate) id: u32,
    pub(crate) padding: Padding<u32, 1>,
    /// if id is [`GpioV2LineAttrId`]::Values, a bitmap containing the
    /// values to which the lines will be set, with each bit number
    /// corresponding to the index into the [`GpioV2LineRequest`].offsets array.
    pub(crate) u: Union,
}

/// A configuration attribute associated with one or more of the requested lines.
#[derive(Debug)]
#[repr(C)]
pub(crate) struct GpioV2LineConfigAttribute {
    /// the configurable attribute
    pub(crate) attr: GpioV2LineAttribute,
    /// a bitmap identifying the lines to which the attribute applies,
    /// with each bit number corresponding to the index into the
    /// [`GpioV2LineRequest`].offsets array.
    pub(crate) mask: libc::c_ulong,
}

/// Configuration for GPIO lines.
#[derive(Debug)]
#[repr(C)]
pub(crate) struct GpioV2LineConfig {
    /// a bitmap containing the flags for the lines,
    /// with values from [`GpioV2LineFlag`].
    pub(crate) flags: libc::c_ulong,
    /// the number of attributes in the `attrs` array.
    pub(crate) num_attrs: u32,
    pub(crate) padding: Padding<u32, 5>,
    /// an array of attributes to configure for the lines.
    pub(crate) attrs: [GpioV2LineConfigAttribute; GPIO_V2_LINE_NUM_ATTRS_MAX],
}

/// Information about a request for GPIO lines.
#[derive(Debug)]
#[repr(C)]
pub(crate) struct GpioV2LineRequest {
    /// an array of desired lines, specified by offset index for the
    /// associated GPIO chip.
    pub(crate) offsets: [u32; GPIO_V2_LINES_MAX],
    /// a desired consumer label for the GPIO line(s).
    pub(crate) consumer: CString<GPIO_MAX_NAME_SIZE>,
    /// the configuration for the requested lines.
    pub(crate) config: GpioV2LineConfig,
    /// the number of lines requested.
    pub(crate) num_lines: u32,
    /// a suggested minimum number of line events that the
    /// kernel should buffer.
    ///
    /// This is only relevant **if edge detection is enabled** in the [`GpioV2LineConfig`].
    ///
    /// Note that this is **only a suggested value** and the kernel may allocate a
    /// larger buffer or cap the size of the buffer.
    ///
    /// If this field is zero then the buffer size defaults to a minimum of `16` events.
    pub(crate) event_buffer_size: u32,
    pub(crate) padding: Padding<u32, 5>,
    /// after a successful `GPIO_V2_GET_LINE_IOCTL` operation, contains
    /// a valid anonymous file descriptor representing the request.
    pub(crate) fd: libc::c_int,
}

/// Information about a certain GPIO line.
#[derive(Debug)]
#[repr(C)]
pub(crate) struct GpioV2LineInfo {
    /// the name of this GPIO line, such as the output pin of the line on
    /// the chip, a rail or a pin header name on a board, as specified by the
    /// GPIO chip, may be empty
    ///
    /// i.e. `name[0] == '\0'`
    pub(crate) name: CString<GPIO_MAX_NAME_SIZE>,
    /// a functional name for the consumer of this GPIO line as set
    /// by whatever is using it, will be empty if there is no current user but
    /// may also be empty if the consumer doesn't set this up.
    pub(crate) consumer: CString<GPIO_MAX_NAME_SIZE>,
    /// the local offset on this GPIO chip, fill this in when requesting
    /// the line information from the kernel
    pub(crate) offset: u32,
    /// the number of attributes in the `attrs` array
    pub(crate) num_attrs: u32,
    /// the flags for this GPIO line, with values from [`GpioV2LineFlag`]
    pub(crate) flags: libc::c_ulong,
    /// the configuration attributes associated with the line
    pub(crate) attrs: [GpioV2LineAttribute; GPIO_V2_LINE_NUM_ATTRS_MAX],
    pub(crate) padding: Padding<u32, 4>,
}

/// [`GpioV2LineInfoChanged`] event type
#[derive(Debug)]
#[repr(u32)]
pub enum GpioV2LineChangedType {
    Requested = 1,
    Released = 2,
    Config = 3,
}

/// Information about a change in status of a GPIO line.
#[derive(Debug)]
#[repr(C)]
pub(crate) struct GpioV2LineInfoChanged {
    /// updated line information.
    pub(crate) info: GpioV2LineInfo,
    /// estimate of time of status change occurrence, in nanoseconds
    pub(crate) timestamp_ns: libc::c_ulong,
    /// the type of change with a value from [`GpioV2LineChangedType`]
    pub(crate) event_type: u32,
    pub(crate) padding: Padding<u32, 5>,
}

/// [`GpioV2LineEvent`] id
#[derive(Debug)]
#[repr(u32)]
pub(crate) enum GpioV2LineEventId {
    RisingEdge = 1,
    FallingEdge = 2,
}

/// The actual event being pushed to userspace.
///
/// By default the `timestamp_ns` is read from `CLOCK_MONOTONIC` and is
/// intended to allow the accurate measurement of the time between events.
/// It does not provide the wall-clock time.
///
/// If the `GPIO_V2_LINE_FLAG_EVENT_CLOCK_REALTIME` flag is set then the
/// `timestamp_ns` is read from `CLOCK_REALTIME`.
///
/// If the `GPIO_V2_LINE_FLAG_EVENT_CLOCK_HTE` flag is set then the
/// `timestamp_ns` is provided by the hardware timestamping engine (HTE) subsystem.
#[derive(Debug)]
#[repr(C)]
pub(crate) struct GpioV2LineEvent {
    /// best estimate of time of event occurrence, in nanoseconds
    pub(crate) timestamp_ns: libc::c_ulong,
    /// the type of event with a value from [`GpioV2LineEventId`]
    pub(crate) id: u32,
    /// the offset of the line that triggered the event
    pub(crate) offset: u32,
    /// the sequence number for this event in the sequence of events for
    /// all the lines in this line request
    pub(crate) seqno: u32,
    /// the sequence number for this event in the sequence of
    /// events on this particular line
    pub(crate) line_seqno: u32,
    pub(crate) padding: Padding<u32, 6>,
}

crate::macros::wrap_ioctl!(
    ioctl_readwrite!(
        gpio_v2_get_lineinfo_ioctl,
        crate::ffi::common::GPIO_IOC_MAGIC,
        0x05,
        crate::ffi::v2::GpioV2LineInfo
    ),
    crate::error::IoctlKind::GetLineInfo
);

crate::macros::wrap_ioctl!(
    ioctl_readwrite!(
        gpio_v2_get_lineinfo_watch_ioctl,
        crate::ffi::common::GPIO_IOC_MAGIC,
        0x06,
        crate::ffi::v2::GpioV2LineInfo
    ),
    crate::error::IoctlKind::GetLineInfo
);

crate::macros::wrap_ioctl!(
    ioctl_readwrite!(
        gpio_v2_get_line_ioctl,
        crate::ffi::common::GPIO_IOC_MAGIC,
        0x07,
        crate::ffi::v2::GpioV2LineRequest
    ),
    crate::error::IoctlKind::GetLine
);

crate::macros::wrap_ioctl!(
    ioctl_readwrite!(
        gpio_v2_line_set_config_ioctl,
        crate::ffi::common::GPIO_IOC_MAGIC,
        0x0D,
        crate::ffi::v2::GpioV2LineConfig
    ),
    crate::error::IoctlKind::SetLineConfig
);

crate::macros::wrap_ioctl!(
    ioctl_readwrite!(
        gpio_v2_line_get_values_ioctl,
        crate::ffi::common::GPIO_IOC_MAGIC,
        0x0E,
        crate::ffi::v2::GpioV2LineValues
    ),
    crate::error::IoctlKind::GetValues
);

crate::macros::wrap_ioctl!(
    ioctl_readwrite!(
        gpio_v2_line_set_values_ioctl,
        crate::ffi::common::GPIO_IOC_MAGIC,
        0x0F,
        crate::ffi::v2::GpioV2LineValues
    ),
    crate::error::IoctlKind::SetValues
);

mod helper {
    use super::*;

    impl From<u32> for GpioV2LineAttrId {
        fn from(value: u32) -> Self {
            debug_assert!(matches!(value, 1..=3));
            match value {
                1 => Self::Flags,
                2 => Self::OutputValues,
                _ => Self::Debounce,
            }
        }
    }

    impl Debug for GpioV2LineAttribute {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let id = GpioV2LineAttrId::from(self.id);
            f.debug_struct("GpioV2LineAttribute")
                .field("id", &id)
                .field(
                    "value",
                    &match id {
                        GpioV2LineAttrId::Flags => unsafe { self.u.flags },
                        GpioV2LineAttrId::OutputValues => unsafe { self.u.values },
                        GpioV2LineAttrId::Debounce => unsafe { self.u.debounce_period_us.into() },
                    },
                )
                .finish()
        }
    }

    impl From<u32> for GpioV2LineChangedType {
        fn from(value: u32) -> Self {
            match value {
                1 => Self::Requested,
                2 => Self::Released,
                _ => Self::Config,
            }
        }
    }
}
