use std::os::fd::AsRawFd;

use crate::{chip::Chip, ffi, line::LineInfo, Result};

#[cfg(feature = "v1")]
pub use ffi::v1::GpioLineChangedType as LineChangedType;
#[cfg(feature = "v2")]
pub use ffi::v2::GpioV2LineChangedType as LineChangedType;

#[repr(transparent)]
pub struct LineInfoChanged {
    #[cfg(feature = "v2")]
    inner: ffi::v2::GpioV2LineInfoChanged,
    #[cfg(feature = "v1")]
    inner: ffi::v1::GpioLineInfoChanged,
}

impl LineInfoChanged {
    pub fn event_type(&self) -> LineChangedType {
        self.inner.event_type.into()
    }

    pub fn lineinfo(&self) -> &LineInfo {
        #[cfg(feature = "v2")]
        {
            unsafe { &*(&self.inner.info as *const ffi::v2::GpioV2LineInfo as *const LineInfo) }
        }
        #[cfg(feature = "v1")]
        {
            unsafe { &*(&self.inner.info as *const ffi::v1::GpioLineInfo as *const LineInfo) }
        }
    }

    pub fn timestamp_ns(&self) -> libc::c_ulong {
        #[cfg(feature = "v2")]
        {
            self.inner.timestamp_ns
        }
        #[cfg(feature = "v1")]
        {
            self.inner.timestamp
        }
    }

    pub fn read(chip: &Chip, buf: &mut [LineInfoChanged]) -> Result<usize> {
        const T_LEN: usize = std::mem::size_of::<LineInfoChanged>();
        let ptr = std::ptr::addr_of_mut!(*buf) as *mut LineInfoChanged as *mut libc::c_void;
        match unsafe { libc::read(chip.file.as_raw_fd(), ptr, T_LEN * 8) } {
            -1 => Err(crate::error::ioctl_error(
                crate::IoctlKind::GetLineEvent,
                nix::Error::last(),
            )),
            n => {
                debug_assert!(n >= 0);
                let n = n.unsigned_abs();
                debug_assert!(n % T_LEN == 0);
                Ok(n)
            }
        }
    }
}

impl Default for LineInfoChanged {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}
