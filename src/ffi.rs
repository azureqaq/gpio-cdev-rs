use bitflags::bitflags;

use crate::IoctlKind;

/// The maximum isze of name and lable arrays
const GPIO_MAX_NAME_SIZE: usize = 32;

/// Infomation about a certain GPIO chip
#[repr(C)]
pub(crate) struct GpioChipInfo {
    /// the Linux kernel name of this GPIO chip
    pub(crate) name: [libc::c_char; GPIO_MAX_NAME_SIZE],
    /// a functional name for this GPIO chip, such as a product number,
    /// may be empty (i.e. `lable[0] == '\0'`)
    pub(crate) lable: [libc::c_char; GPIO_MAX_NAME_SIZE],
    /// number of GPIO lines on this chip
    pub(crate) lines: u32,
}

/// Maximum number of requested lines
#[cfg(feature = "v2")]
const GPIO_V2_LINES_MAX: usize = 64;

/// The maximum number of configuration attributes associated with a line request
#[cfg(feature = "v2")]
const GPIO_V2_LINE_NUM_ATTRS_MAX: usize = 10;

#[cfg(feature = "v2")]
bitflags! {
    /// struct `GpioV2LineAttribute.flags` values
    pub struct GpioV2LineFlag: libc::c_ulong {
        /// line is not available for request
        const GPIO_V2_LINE_FLAG_USED                 = 1 << 0;

        /// line active state is physical low
        const GPIO_V2_LINE_FLAG_ACTIVE_LOW           = 1 << 1;

        /// line is an input
        const GPIO_V2_LINE_FLAG_INPUT                = 1 << 2;

        /// line is an output
        const GPIO_V2_LINE_FLAG_OUTPUT               = 1 << 3;

        /// line detects rising (inactive to active) edges
        const GPIO_V2_LINE_FLAG_EDGE_RISING          = 1 << 4;

        /// line detects falling (active to inactive) edges
        const GPIO_V2_LINE_FLAG_EDGE_FALLING         = 1 << 5;

        /// line is an open drain output
        const GPIO_V2_LINE_FLAG_OPEN_DRAIN           = 1 << 6;

        /// line is an open source output
        const GPIO_V2_LINE_FLAG_OPEN_SOURCE          = 1 << 7;

        /// line has pull-up bias enabled
        const GPIO_V2_LINE_FLAG_BIAS_PULL_UP         = 1 << 8;

        /// line has pull-down bias enabled
        const GPIO_V2_LINE_FLAG_BIAS_PULL_DOWN       = 1 << 9;

        /// line has bias disabled
        const GPIO_V2_LINE_FLAG_BIAS_DISABLED        = 1 << 10;

        /// line events contain REALTIME timestamps
        const GPIO_V2_LINE_FLAG_EVENT_CLOCK_REALTIME = 1 << 11;

        /// line events contain timestamps from hardware timestamp engine
        const GPIO_V2_LINE_FLAG_EVENT_CLOCK_HTE      = 1 << 12;
    }
}

/// Values of GPIO lines
#[repr(C)]
#[cfg(feature = "v2")]
pub(crate) struct GpioV2LineValues {
    /// a bitmap containing the value of the lines, set to 1 for active and 0 for inactive
    bits: libc::c_ulong,
    /// a bitmap identifying the lines to get or set, with each bit number corresponding
    /// to the index into `GpioV2LineRequest.offsets`
    mask: libc::c_ulong,
}

#[cfg(feature = "v2")]
bitflags! {
    /// `&GpioV2LineAttribute.id` values
    /// identifying which field of the attribute union is in use
    pub(crate) struct GpioV2LineAttrId: u32 {
        /// flags field is in use
        const GPIO_V2_LINE_ATTR_ID_FLAGS         = 1;

        /// values field is in use
        const GPIO_V2_LINE_ATTR_ID_OUTPUT_VALUES = 2;

        /// debounce_period_us field is in use
        const GPIO_V2_LINE_ATTR_ID_DEBOUNCE      = 3;
    }
}

#[repr(C)]
pub(crate) union Union {
    /// if id is `GPIO_V2_LINE_ATTR_ID_FLAGS`, the flags for the GPIO
    /// line, with values from `GpioV2LineFlag`, such as
    /// `GPIO_V2_LINE_FLAG_ACTIVE_LOW`, `GPIO_V2_LINE_FLAG_OUTPUT` etc,
    /// added together.
    ///
    /// This overrides the default flags contained in the
    /// `GpioV2LineConfig` for the associated line.
    flags: libc::c_ulong,
    /// if id is `GPIO_V2_LINE_ATTR_ID_OUTPUT_VALUES`, a bitmap
    /// containing the values to which the lines will be set, with each bit
    /// number corresponding to the index into
    /// `GpioV2LineRequest.offsets`.
    values: libc::c_ulong,
    /// if id is `GPIO_V2_LINE_ATTR_ID_DEBOUNCE`, the
    /// desired debounce period, in microseconds.
    debounce_period_us: u32,
}

/// A configurable attribute of a line
#[repr(C)]
#[cfg(feature = "v2")]
pub(crate) struct GpioV2LineAttribute {
    /// attribute identifier with value from `GpioV2LineAttrId`.
    pub(crate) id: u32,
    /// reserved for future use and must be zero filled.
    pub(crate) padding: u32,
    pub(crate) u: Union,
}

/// A configuration attribute associated with one or more of the requested lines
#[repr(C)]
#[cfg(feature = "v2")]
pub(crate) struct GpioV2LineConfigAttribute {
    /// the configurable attribute
    pub(crate) attr: GpioV2LineAttribute,
    /// a bitmap identifying the lines to which the attribute applies,
    /// with each bit number corresponding to the index into `GpioV2LineRequest.offsets`.
    pub(crate) mask: libc::c_ulong,
}

/// Configuration for GPIO lines
#[repr(C)]
#[cfg(feature = "v2")]
pub(crate) struct GpioV2LineConfig {
    /// flags for the GPIO lines, with values from gpio_v2_line_flag, such as
    /// `GPIO_V2_LINE_FLAG_ACTIVE_LOW`, `GPIO_V2_LINE_FLAG_OUTPUT` etc,
    /// added together.
    ///
    /// This is the default for all requested lines.
    /// but may be overridden for particular lines using.
    pub(crate) flags: libc::c_ulong,
    /// the number of attributes in `attrs`.
    pub(crate) num_attrs: u32,
    /// reserved for future use and must be zero filled.
    pub(crate) padding: [u32; 5],
    /// the configuration attributes associated with the requested
    /// lines. Any attribute should only be associated with a particular line
    /// once. If an attribute is associated with a line multiple times then the
    /// first occurrence (i.e. lowest index) has precedence.
    pub(crate) attrs: [GpioV2LineConfigAttribute; GPIO_V2_LINE_NUM_ATTRS_MAX],
}

/// Information about a request for GPIO lines
#[repr(C)]
#[cfg(feature = "v2")]
pub(crate) struct GpioV2LineRequest {
    /// an array of desired lines, specified by offset index for the associated GPIO chip
    pub(crate) offsets: [u32; GPIO_V2_LINES_MAX],
    /// a desired consumer label for the selected GPIO lines such as `"my-bitbanged-relay"`
    pub(crate) consumer: [libc::c_char; GPIO_MAX_NAME_SIZE],
    /// requested configuration for the lines
    pub(crate) config: GpioV2LineConfig,
    /// number of lines requested in this request, i.e. the number of valid fields in the
    /// `GPIO_V2_LINES_MAX` sized arrays, set to 1 to request a single line
    pub(crate) num_lines: u32,
    /// a suggested minimum number of line events that the kernel should buffer.
    ///
    /// This is only relevant if edge detection is enabled in the configuration.
    ///
    /// Note that this is only a suggested value and the kernel may allocate a
    /// larger buffer or cap the size of the buffer.
    ///
    /// If this field is zero then the buffer size defaults to a minimum of `num_lines * 16`.
    pub(crate) event_buffer_size: u32,
    /// reserved for future use and must be zero filled
    pub(crate) padding: [u32; 5],
    /// if successful this field will contain a valid anonymous file handle
    /// after a `GPIO_GET_LINE_IOCTL` operation, zero or negative value means error.
    pub(crate) fd: u32,
}

/// Information about a certain GPIO line
#[repr(C)]
#[cfg(feature = "v2")]
pub(crate) struct GpioV2LineInfo {
    /// the name of this GPIO line, such as the output pin of the line on
    /// the chip, a rail or a pin header name on a board, as specified by the
    /// GPIO chip, may be empty (i.e. name[0] == '\0')
    pub(crate) name: [libc::c_char; GPIO_MAX_NAME_SIZE],
    /// a functional name for the consumer of this GPIO line as set
    /// by whatever is using it, will be empty if there is no current user but
    /// may also be empty if the consumer doesn't set this up
    pub(crate) consumer: [libc::c_char; GPIO_MAX_NAME_SIZE],
    /// the local offset on this GPIO chip, fill this in when
    /// requesting the line information from the kernel
    pub(crate) offset: u32,
    /// the number of attributes in `attrs`
    pub(crate) num_attrs: u32,
    /// flags for this GPIO line, with values from `GpioV2LineFlag`,
    /// such as `GPIO_V2_LINE_FLAG_ACTIVE_LOW`, `GPIO_V2_LINE_FLAG_OUTPUT` etc,
    /// added together.
    pub(crate) flags: libc::c_ulong,
    /// the configuration attributes associated with the line
    pub(crate) attrs: [GpioV2LineAttribute; GPIO_V2_LINE_NUM_ATTRS_MAX],
    /// reserved for future use
    pub(crate) padding: [u32; 4],
}

#[cfg(feature = "v2")]
bitflags! {
    /// `GpioV2LineChanged.event_type` values
    pub(crate) struct GpioV2LineChangedType: u32 {
        /// line has been requested
        const GPIO_V2_LINE_CHANGED_REQUESTED = 1;
        /// line has been released
        const GPIO_V2_LINE_CHANGED_RELEASED  = 2;
        /// line has been reconfigured
        const GPIO_V2_LINE_CHANGED_CONFIG    = 3;
    }
}

/// Information about a change in status of a GPIO line
#[repr(C)]
#[cfg(feature = "v2")]
pub(crate) struct GpioV2LineInfoChanged {
    /// updated line information
    pub(crate) info: GpioV2LineInfo,
    /// estimate of time of status change occurrence, in nanoseconds
    pub(crate) timestamp_ns: libc::c_ulong,
    /// the type of change with a value from `GpioV2LineChangedType`
    pub(crate) event_type: u32,
    /// reserved for future use
    pub(crate) padding: [u32; 5],
}

#[cfg(feature = "v2")]
bitflags! {
    /// `GpioV2LineEvent.id` values
    pub(crate) struct GpioV2LineEventId: u32 {
        /// event triggered by a rising edge
        const GPIO_V2_LINE_EVENT_RISING_EDGE  = 1;

        /// event triggered by a falling edge
        const GPIO_V2_LINE_EVENT_FALLING_EDGE = 2;
    }
}

/// The actual event being pushed to userspace
///
/// By default the `timestamp_ns` is read from `CLOCK_MONOTONIC` and is
/// intended to allow the accurate measurement of the time between events.
/// It does not provide the wall-clock time.
///
/// If the `GPIO_V2_LINE_FLAG_EVENT_CLOCK_REALTIME` flag is set then the
/// `timestamp_ns` is read from `CLOCK_REALTIME`.
#[repr(C)]
#[cfg(feature = "v2")]
pub(crate) struct GpioV2LineEvent {
    /// best estimate of time of event occurrence, in nanoseconds
    pub(crate) timestamp_ns: libc::c_ulong,
    /// event identifier with value from `GpioV2LineEventId`
    pub(crate) id: u32,
    /// the offset of the line that triggered the event
    pub(crate) offset: u32,
    /// the sequence number for this event in the sequence of events for
    /// all the lines in this line request
    pub(crate) seqno: u32,
    /// the sequence number for this event in the sequence of
    /// events on this particular line
    pub(crate) line_seqno: u32,
    /// reserved for future use
    pub(crate) padding: [u32; 6],
}

// ABI v1
// This version of the ABI is deprecated.
// Use the latest version of the ABI, defined above, instead.

#[cfg(feature = "v1")]
bitflags! {
    /// Informational flags
    pub(crate) struct GpioLineFlag: u32 {
        const GPIOLINE_FLAG_KERNEL         = 1 << 0;
        const GPIOLINE_FLAG_IS_OUT         = 1 << 1;
        const GPIOLINE_FLAG_ACTIVE_LOW     = 1 << 2;
        const GPIOLINE_FLAG_OPEN_DRAIN     = 1 << 3;
        const GPIOLINE_FLAG_OPEN_SOURCE    = 1 << 4;
        const GPIOLINE_FLAG_BIAS_PULL_UP   = 1 << 5;
        const GPIOLINE_FLAG_BIAS_PULL_DOWN = 1 << 6;
        const GPIOLINE_FLAG_BIAS_DISABLE   = 1 << 7;
    }
}

/// Information about a certain GPIO line
#[cfg(feature = "v1")]
#[repr(C)]
pub(crate) struct GpioLineInfo {
    /// the local offset on this GPIO device, fill this in when
    /// requesting the line information from the kernel
    line_offset: u32,
    /// various flags for this line
    flags: u32,
    /// the name of this GPIO line, such as the output pin of the line on the
    /// chip, a rail or a pin header name on a board, as specified by the gpio
    /// chip, may be empty (i.e. `name[0] == '\0'`)
    name: [libc::c_char; GPIO_MAX_NAME_SIZE],
    /// a functional name for the consumer of this GPIO line as set by
    /// whatever is using it, will be empty if there is no current user but may
    /// also be empty if the consumer doesn't set this up
    consumer: [libc::c_char; GPIO_MAX_NAME_SIZE],
}

/// Maximum number of requested handles
#[cfg(feature = "v1")]
const GPIOHANDLES_MAX: usize = 64;

/// Possible line status change events
#[cfg(feature = "v1")]
#[repr(u32)]
pub(crate) enum GpioLineChanged {
    GpioLineChangedRequested = 1,
    GpioLineChangedReleased,
    GpioLineChangedConfig,
}

/// Information about a change in status
#[cfg(feature = "v1")]
#[repr(C)]
pub(crate) struct GpioLineInfoChanged {
    /// updated line information
    pub(crate) info: GpioLineInfo,
    /// estimate of time of status change occurrence, in nanoseconds
    pub(crate) timestamp: u64,
    /// `GpioLineChanged`
    pub(crate) event_type: u32,
    /// reserved for future use
    pub(crate) padding: [u32; 5],
}

/// Linerequest flags
#[cfg(feature = "v1")]
bitflags! {
    pub(crate) struct GpioHandleRequestFlags: u32 {
        const GPIOHANDLE_REQUEST_INPUT          = 1 << 0;
        const GPIOHANDLE_REQUEST_OUTPUT         = 1 << 1;
        const GPIOHANDLE_REQUEST_ACTIVE_LOW     = 1 << 2;
        const GPIOHANDLE_REQUEST_OPEN_DRAIN     = 1 << 3;
        const GPIOHANDLE_REQUEST_OPEN_SOURCE    = 1 << 4;
        const GPIOHANDLE_REQUEST_BIAS_PULL_UP   = 1 << 5;
        const GPIOHANDLE_REQUEST_BIAS_PULL_DOWN = 1 << 6;
        const GPIOHANDLE_REQUEST_BIAS_DISABLE   = 1 << 7;
    }
}

/// Information about a GPIO handle request
#[cfg(feature = "v1")]
#[repr(C)]
pub(crate) struct GpioHandleRequest {
    /// an array of desired lines, specified by offset index for the
    /// associated GPIO device
    pub(crate) lineoffsets: [u32; GPIOHANDLES_MAX],
    /// desired flags for the desired GPIO lines, `GpioHandleRequestFlags`
    pub(crate) flags: u32,
    /// if the `GPIOHANDLE_REQUEST_OUTPUT` is set for a requested
    /// line, this specifies the default output value, should be 0 (low) or
    /// 1 (high), anything else than 0 or 1 will be interpreted as 1 (high)
    pub(crate) default_values: [u8; GPIOHANDLES_MAX],
    /// a desired consumer label for the selected GPIO line(s)
    /// such as "my-bitbanged-relay"
    pub(crate) consumer_lable: [libc::c_char; GPIO_MAX_NAME_SIZE],
    /// number of lines requested in this request, i.e. the number of
    /// valid fields in the above arrays, set to 1 to request a single line
    pub(crate) lines: u32,
    /// if successful this field will contain a valid anonymous file handle
    /// after a `GPIO_GET_LINEHANDLE_IOCTL` operation, zero or negative value
    /// means error
    pub(crate) fd: libc::c_int,
}

/// Configuration for a GPIO handle request
#[cfg(feature = "v1")]
#[repr(C)]
pub(crate) struct GpioHandleConfig {
    /// updated flags for the requested GPIO lines, `GpioHandleRequestFlags`
    pub(crate) flags: u32,
    /// if the %GPIOHANDLE_REQUEST_OUTPUT is set in flags,
    /// this specifies the default output value, should be 0 (low) or
    /// 1 (high), anything else than 0 or 1 will be interpreted as 1 (high)
    pub(crate) default_values: [u8; GPIOHANDLES_MAX],
    /// reserved for future use and should be zero filled
    pub(crate) padding: [u32; 4],
}

/// Information of values on a GPIO handle
#[cfg(feature = "v1")]
#[repr(C)]
pub(crate) struct GpioHandleData {
    /// when getting the state of lines this contains the current
    /// state of a line, when setting the state of lines these should contain
    /// the desired target state
    pub(crate) values: [u8; GPIOHANDLES_MAX],
}

#[cfg(feature = "v1")]
bitflags! {
    pub(crate) struct GpioEventRequestFlags: u32 {
        const GPIOEVENT_REQUEST_RISING_EDGE = 1 << 0;
        const GPIOEVENT_REQUEST_FALLING_EDGE = 1 << 1;
        const GPIOEVENT_REQUEST_BOTH_EDGES = (1 << 0) | (1 << 1);
    }
}

/// Information about a GPIO event request
#[cfg(feature = "v1")]
#[repr(C)]
pub(crate) struct GpioEventRequest {
    /// the desired line to subscribe to events from, specified by
    /// offset index for the associated GPIO device
    pub(crate) lineoffset: u32,
    /// desired handle flags for the desired GPIO line, `GpioHandleRequestFlags`
    pub(crate) handleflags: u32,
    /// desired flags for the desired GPIO event line, `GpioEventRequestFlags`
    pub(crate) eventflags: u32,
    /// a desired consumer label for the selected GPIO line(s) such as "my-listener"
    pub(crate) consumer_label: [libc::c_char; GPIO_MAX_NAME_SIZE],
    /// if successful this field will contain a valid anonymous file handle
    /// after a `GPIO_GET_LINEEVENT_IOCTL` operation, zero or negative value
    /// means error
    pub(crate) fd: libc::c_int,
}

/// GPIO event types
#[repr(u32)]
pub(crate) enum GpioEventEdges {
    GpioeventEventResingEdge = 0x01,
    GpioeventEventFallingEdge = 0x02,
}

/// The actual event being pushed to userspace
#[cfg(feature = "v1")]
#[repr(C)]
pub(crate) struct GpioEventData {
    /// best estimate of time of event occurrence, in nanoseconds
    pub(crate) timestamp: u64,
    /// event identifier, `GpioEventEdges`
    pub(crate) id: u32,
}

macro_rules! wrap_ioctl {
    ($ioctl_macro:ident!($name:ident, $ioty:expr, $nr: expr, $ty:ident), $ioctl_error_type:expr) => {
        mod $name {
            #[allow(unused)]
            use super::*;
            nix::$ioctl_macro!($name, $ioty, $nr, $ty);
        }

        pub(crate) fn $name(fd: libc::c_int, data: &mut $ty) -> crate::errors::Result<libc::c_int> {
            unsafe {
                $name::$name(fd, data).map_err(|e| crate::errors::ioctl_err($ioctl_error_type, e))
            }
        }
    };
}

wrap_ioctl!(
    ioctl_read!(gpio_get_chipinfo_ioctl, 0xB4, 0x01, GpioChipInfo),
    IoctlKind::ChipInfo
);

wrap_ioctl!(
    ioctl_readwrite!(gpio_get_lineinfo_unwatch_ioctl, 0xB4, 0x0C, u32),
    IoctlKind::LineInfo
);

#[cfg(feature = "v2")]
wrap_ioctl!(
    ioctl_readwrite!(gpio_v2_get_lineinfo_ioctl, 0xB4, 0x05, GpioV2LineInfo),
    IoctlKind::LineInfo
);

#[cfg(feature = "v2")]
wrap_ioctl!(
    ioctl_readwrite!(
        gepio_v2_get_lineinfo_watch_ioctl,
        0xB4,
        0x06,
        GpioV2LineInfo
    ),
    IoctlKind::LineInfo
);

#[cfg(feature = "v2")]
wrap_ioctl!(
    ioctl_readwrite!(gepio_v2_get_line_ioctl, 0xB4, 0x07, GpioV2LineRequest),
    IoctlKind::LineHandle
);

#[cfg(feature = "v2")]
wrap_ioctl!(
    ioctl_readwrite!(gepio_v2_line_set_config_ioctl, 0xB4, 0x0D, GpioV2LineConfig),
    IoctlKind::SetLine
);

#[cfg(feature = "v2")]
wrap_ioctl!(
    ioctl_readwrite!(gepio_v2_line_get_values_ioctl, 0xB4, 0x0E, GpioV2LineValues),
    IoctlKind::GetLine
);

#[cfg(feature = "v2")]
wrap_ioctl!(
    ioctl_readwrite!(gepio_v2_line_set_values_ioctl, 0xB4, 0x0F, GpioV2LineValues),
    IoctlKind::SetLine
);

#[cfg(feature = "v1")]
wrap_ioctl!(
    ioctl_readwrite!(gpio_get_lineinfo_ioctl, 0xB4, 0x02, GpioLineInfo),
    IoctlKind::LineInfo
);

#[cfg(feature = "v1")]
wrap_ioctl!(
    ioctl_readwrite!(gpio_get_linehandle_ioctl, 0xB4, 0x03, GpioHandleRequest),
    IoctlKind::GetLine
);

#[cfg(feature = "v1")]
wrap_ioctl!(
    ioctl_readwrite!(gpio_get_lineevent_ioctl, 0xB4, 0x04, GpioEventRequest),
    IoctlKind::LineEvent
);

#[cfg(feature = "v1")]
wrap_ioctl!(
    ioctl_readwrite!(gpiohandle_get_line_values_ioctl, 0xB4, 0x08, GpioHandleData),
    IoctlKind::GetLine
);

#[cfg(feature = "v1")]
wrap_ioctl!(
    ioctl_readwrite!(gpiohandle_set_line_values_ioctl, 0xB4, 0x09, GpioHandleData),
    IoctlKind::SetLine
);

#[cfg(feature = "v1")]
wrap_ioctl!(
    ioctl_readwrite!(gpiohandle_set_config_ioctl, 0xB4, 0x0A, GpioHandleConfig),
    IoctlKind::LineHandle
);

#[cfg(feature = "v1")]
wrap_ioctl!(
    ioctl_readwrite!(gpio_get_lineinfo_watch_ioctl, 0xB4, 0x0B, GpioLineInfo),
    IoctlKind::LineInfo
);
