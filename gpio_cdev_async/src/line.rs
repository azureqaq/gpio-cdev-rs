use std::{borrow::Cow, fmt::Debug, os::fd::OwnedFd};

use crate::ffi;

#[cfg(feature = "v1")]
pub use ffi::v1::GpioHandleFlags as HandleFlags;
#[cfg(feature = "v2")]
pub use ffi::v2::GpioV2LineFlag as HandleFlags;

#[cfg(feature = "v1")]
pub use ffi::v1::GpioLineFlag as LineFlags;
#[cfg(feature = "v2")]
pub use ffi::v2::GpioV2LineFlag as LineFlags;

use tinyvec::TinyVec;

#[repr(transparent)]
pub struct LineInfo {
    #[cfg(feature = "v1")]
    pub(crate) inner: ffi::v1::GpioLineInfo,
    #[cfg(feature = "v2")]
    pub(crate) inner: ffi::v2::GpioV2LineInfo,
}

impl LineInfo {
    pub fn offset(&self) -> u32 {
        #[cfg(feature = "v1")]
        {
            self.inner.line_offset
        }

        #[cfg(feature = "v2")]
        {
            self.inner.offset
        }
    }

    pub fn flags(&self) -> LineFlags {
        LineFlags::from_bits_retain(self.inner.flags)
    }

    pub fn consumer(&self) -> Cow<'_, str> {
        self.inner.consumer.to_string_lossy()
    }

    pub fn name(&self) -> Cow<'_, str> {
        self.inner.name.to_string_lossy()
    }

    #[cfg(feature = "v2")]
    pub fn num_attrs(&self) -> u32 {
        self.inner.num_attrs
    }

    #[cfg(feature = "v2")]
    pub fn attrs(&self) -> tinyvec::ArrayVec<[LineAttribute; ffi::v2::GPIO_V2_LINE_NUM_ATTRS_MAX]> {
        debug_assert!(self.num_attrs() as usize <= ffi::v2::GPIO_V2_LINE_NUM_ATTRS_MAX);
        self.inner
            .attrs
            .iter()
            .take(self.num_attrs() as usize)
            .map(LineAttribute::from)
            .collect()
    }
}

impl Debug for LineInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut temp = f.debug_struct("LineInfo");
        temp.field("offset", &self.offset());
        temp.field("flags", &self.flags());
        temp.field("name", &self.name());
        temp.field("consumer", &self.consumer());
        #[cfg(feature = "v2")]
        temp.field("attrs", &self.attrs());
        temp.finish()
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg(feature = "v2")]
pub enum LineAttribute {
    Flags(LineFlags),
    Values(libc::c_ulong),
    DebouncePeriodUs(u32),
}

#[cfg(feature = "v2")]
impl From<&ffi::v2::GpioV2LineAttribute> for LineAttribute {
    fn from(attr: &ffi::v2::GpioV2LineAttribute) -> Self {
        use ffi::v2::GpioV2LineAttrId;
        let id = GpioV2LineAttrId::from(attr.id);
        match id {
            GpioV2LineAttrId::Flags => {
                Self::Flags(LineFlags::from_bits_retain(unsafe { attr.u.flags }))
            }
            GpioV2LineAttrId::OutputValues => Self::Values(unsafe { attr.u.values }),
            GpioV2LineAttrId::Debounce => {
                Self::DebouncePeriodUs(unsafe { attr.u.debounce_period_us })
            }
        }
    }
}

#[cfg(feature = "v2")]
impl Default for LineAttribute {
    /// This implementation is solely to meet the requirements of `tinyvec`.
    /// Do not use it.
    fn default() -> Self {
        Self::Values(0)
    }
}

#[derive(Debug)]
pub struct LinesHandle {
    offsets: TinyVec<[u32; 8]>,
    req_fd: OwnedFd,
}

impl LinesHandle {
    pub fn offsets(&self) -> &[u32] {
        &self.offsets
    }
}

#[repr(transparent)]
pub struct LinesRequest {
    #[cfg(feature = "v1")]
    inner: ffi::v1::GpioHandleRequest,
    #[cfg(feature = "v2")]
    inner: ffi::v2::GpioV2LineRequest,
}

impl LinesRequest {
    pub fn offsets(&self) -> &[u32] {
        #[cfg(feature = "v1")]
        {
            self.inner
                .lineoffsets
                .get(..self.inner.lines as usize)
                .unwrap_or_default()
        }
        #[cfg(feature = "v2")]
        {
            self.inner
                .offsets
                .get(..self.inner.num_lines as usize)
                .unwrap_or_default()
        }
    }

    pub fn consumer(&self) -> Cow<'_, str> {
        #[cfg(feature = "v1")]
        {
            self.inner.consumer_label.to_string_lossy()
        }
        #[cfg(feature = "v2")]
        {
            self.inner.consumer.to_string_lossy()
        }
    }

    pub fn flags(&self) -> HandleFlags {
        #[cfg(feature = "v1")]
        {
            HandleFlags::from_bits_retain(self.inner.flags)
        }
        #[cfg(feature = "v2")]
        {
            HandleFlags::from_bits_retain(self.inner.config.flags)
        }
    }

    pub fn num_lines(&self) -> u32 {
        #[cfg(feature = "v1")]
        {
            self.inner.lines
        }
        #[cfg(feature = "v2")]
        {
            self.inner.num_lines
        }
    }

    #[cfg(feature = "v2")]
    pub fn flags_of_offset(&self, offset: u32) -> Option<HandleFlags> {
        use ffi::v2::{GpioV2LineAttrId, GpioV2LineAttribute};
        let index = self.offsets().iter().position(|&o| o == offset)?;
        let f = self
            .inner
            .config
            .attrs
            .iter()
            .take(self.inner.config.num_attrs as usize)
            .find_map(|c_attr| {
                if c_attr.mask & (1 << index) != 0 {
                    match LineAttribute::from(&c_attr.attr) {
                        LineAttribute::Flags(f) => Some(f),
                        _ => None,
                    }
                } else {
                    None
                }
            });

        Some(f.unwrap_or_else(|| self.flags()))
    }

    #[cfg(feature = "v2")]
    pub fn attr_iter(&self) -> LinesAttrsIter<'_> {
        LinesAttrsIter {
            index: 0,
            offsets: self.offsets(),
            default_flags: self.flags(),
            inner: self.inner.config.attrs.iter(),
        }
    }
}

#[cfg(feature = "v2")]
pub struct LinesAttrsIter<'a> {
    index: usize,
    offsets: &'a [u32],
    default_flags: LineFlags,
    inner: std::slice::Iter<'a, ffi::v2::GpioV2LineConfigAttribute>,
}

#[cfg(feature = "v2")]
#[derive(Debug, Clone, Copy)]
pub struct LinesAttrsIterItem {
    offset: u32,
    attr: LineAttribute,
}

#[cfg(feature = "v2")]
impl Iterator for LinesAttrsIter<'_> {
    type Item = LinesAttrsIterItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.offsets.len() {
            return None;
        }
        let f = self.inner.next().map(|c_attr| {
            if c_attr.mask & (1 << self.index) != 0 {
                LineAttribute::from(&c_attr.attr)
            } else {
                LineAttribute::Flags(self.default_flags)
            }
        });
        let res = f.map(|attr| LinesAttrsIterItem {
            offset: self.offsets[self.index],
            attr,
        });
        self.index += 1;
        res
    }
}
