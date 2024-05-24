use std::{borrow::Cow, fmt::Debug};

use crate::ffi;

#[cfg(feature = "v1")]
pub use ffi::v1::GpioLineFlag as LineFlag;

#[cfg(feature = "v2")]
pub use ffi::v2::GpioV2LineFlag as LineFlag;

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

    pub fn flags(&self) -> LineFlag {
        LineFlag::from_bits_retain(self.inner.flags)
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
            .map(|attr| {
                use ffi::v2::GpioV2LineAttrId;
                let id = GpioV2LineAttrId::from(attr.id);
                match id {
                    GpioV2LineAttrId::Flags => LineAttribute::Flags(unsafe { attr.u.flags }),
                    GpioV2LineAttrId::OutputValues => {
                        LineAttribute::Values(unsafe { attr.u.values })
                    }
                    GpioV2LineAttrId::Debounce => {
                        LineAttribute::DebouncePeriodUs(unsafe { attr.u.debounce_period_us })
                    }
                }
            })
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(feature = "v2")]
pub enum LineAttribute {
    Flags(libc::c_ulong),
    Values(libc::c_ulong),
    DebouncePeriodUs(u32),
}

#[cfg(feature = "v2")]
impl Default for LineAttribute {
    /// This implementation is solely to meet the requirements of `tinyvec`.
    /// Do not use it.
    fn default() -> Self {
        Self::Values(0)
    }
}
