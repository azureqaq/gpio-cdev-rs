use std::os::fd::{AsRawFd, BorrowedFd};

use crate::{chip::Chip, ffi, line::LineInfo, Result};

#[cfg(feature = "v1")]
pub use ffi::v1::GpioLineChangedType as LineChangedType;
#[cfg(feature = "v2")]
pub use ffi::v2::GpioV2LineChangedType as LineChangedType;

#[derive(Debug)]
#[repr(transparent)]
pub struct LineInfoChangedEvent {
    #[cfg(feature = "v2")]
    inner: ffi::v2::GpioV2LineInfoChanged,
    #[cfg(feature = "v1")]
    inner: ffi::v1::GpioLineInfoChanged,
}

impl LineInfoChangedEvent {
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

    pub fn read(chip: &Chip, buf: &mut [LineInfoChangedEvent]) -> Result<usize> {
        const T_LEN: usize = std::mem::size_of::<LineInfoChangedEvent>();
        let ptr = std::ptr::addr_of_mut!(*buf) as *mut LineInfoChangedEvent as *mut libc::c_void;
        match unsafe { libc::read(chip.file.as_raw_fd(), ptr, T_LEN * buf.len()) } {
            -1 => Err(crate::error::ioctl_error(
                crate::IoctlKind::GetLineEvent,
                nix::Error::last(),
            )),
            n => {
                debug_assert!(n > 0);
                let n = n.unsigned_abs();
                debug_assert!(n % T_LEN == 0);
                Ok(n / T_LEN)
            }
        }
    }
}

impl Default for LineInfoChangedEvent {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

pub struct LineInfoChangeIter<'a> {
    chip: &'a Chip,
}

impl Iterator for LineInfoChangeIter<'_> {
    type Item = Result<LineInfoChangedEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        const BUF_SIZE: usize = 1;
        let mut buf = [LineInfoChangedEvent::default(); BUF_SIZE];

        match LineInfoChangedEvent::read(self.chip, &mut buf) {
            Ok(_len) => {
                debug_assert_eq!(_len, BUF_SIZE);
                Some(Ok(buf.into_iter().next().unwrap()))
            }
            Err(e) => Some(Err(e)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}
