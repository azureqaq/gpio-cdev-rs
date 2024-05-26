use std::{
    borrow::Cow,
    fmt::Debug,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
};

use crate::{chip::Chip, ffi, Result};

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
        temp.field("attrs", &self.attrs().as_slice());
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

pub struct LinesHandle {
    offsets: TinyVec<[u32; 8]>,
    req_fd: OwnedFd,
}

impl Debug for LinesHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinesHandle")
            .field("offsets", &self.offsets.as_slice())
            .field("req_fd", &self.req_fd)
            .finish()
    }
}

impl LinesHandle {
    pub fn offsets(&self) -> &[u32] {
        &self.offsets
    }

    pub fn index_of_offset(&self, offset: u32) -> Option<usize> {
        index_of_offset(&self.offsets, offset)
    }

    pub fn get_values(&self) -> Result<LineValues> {
        #[cfg(feature = "v1")]
        {
            let mut data: ffi::v1::GpioHandleData = unsafe { std::mem::zeroed() };
            ffi::v1::gpiohandle_get_line_values_ioctl(self.req_fd.as_raw_fd(), &mut data)?;
            Ok(LineValues {
                inner: data,
                offsets: self.offsets.clone(),
            })
        }
        #[cfg(feature = "v2")]
        {
            let mut mask = 0;
            for index in 0..self.offsets.len() {
                mask |= (1 << index);
            }
            self.get_values_by_mask(mask)
        }
    }

    #[cfg(feature = "v2")]
    pub fn get_values_by_mask(&self, mask: libc::c_ulong) -> Result<LineValues> {
        let mut data: ffi::v2::GpioV2LineValues = unsafe { std::mem::zeroed() };
        data.mask = mask;
        ffi::v2::gpio_v2_line_get_values_ioctl(self.req_fd.as_raw_fd(), &mut data)?;
        Ok(LineValues {
            inner: data,
            offsets: self.offsets.clone(),
        })
    }

    #[cfg(feature = "v2")]
    pub fn get_values_by_offsets(&self, offsets: impl AsRef<[u32]>) -> Result<LineValues> {
        let mask = offsets_to_mask(self.offsets(), offsets);
        self.get_values_by_mask(mask)
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
    pub fn builder() -> LinesRequestBuilder {
        LinesRequestBuilder::new()
    }

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

    pub fn index_of_offset(&self, offset: u32) -> Option<usize> {
        self.offsets().iter().position(|&o| o == offset)
    }

    #[cfg(feature = "v2")]
    fn attrs(&self) -> &[ffi::v2::GpioV2LineConfigAttribute] {
        self.inner
            .config
            .attrs
            .get(..self.inner.config.num_attrs as usize)
            .unwrap_or_default()
    }

    #[cfg(feature = "v2")]
    pub fn flags_of_offset(&self, offset: u32) -> Option<HandleFlags> {
        use ffi::v2::{GpioV2LineAttrId, GpioV2LineAttribute};
        let index = self.index_of_offset(offset)?;
        let f = self.attrs().iter().find_map(|c_attr| {
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

    #[cfg(feature = "v1")]
    pub fn default_values(&self) -> &[u8] {
        self.inner
            .default_values
            .get(..self.inner.lines as usize)
            .unwrap_or_default()
    }

    /// NOT Consider flags OUTPUT
    pub fn default_value_of_offset(&self, offset: u32) -> Option<u8> {
        #[cfg(feature = "v1")]
        {
            let index = self.index_of_offset(offset)?;
            self.default_values().get(index).copied()
        }
        #[cfg(feature = "v2")]
        {
            let index = self.index_of_offset(offset)?;
            self.attrs().iter().find_map(|c_attr| {
                if c_attr.mask & (1 << index) != 0 {
                    if let LineAttribute::Values(values) = LineAttribute::from(&c_attr.attr) {
                        if values & (1 << index) != 0 {
                            Some(1)
                        } else {
                            Some(0)
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        }
    }
}

impl LinesRequest {
    pub fn request(self, chip: &Chip) -> Result<LinesHandle> {
        #[cfg(feature = "v2")]
        {
            let mut data = self;
            ffi::v2::gpio_v2_get_line_ioctl(chip.file.as_raw_fd(), &mut data.inner)?;
            Ok(LinesHandle {
                offsets: data.offsets().into(),
                req_fd: unsafe { OwnedFd::from_raw_fd(data.inner.fd) },
            })
        }
        #[cfg(feature = "v1")]
        {
            let mut data = self;
            ffi::v1::gpio_get_linehandle_ioctl(chip.file.as_raw_fd(), &mut data.inner)?;
            Ok(LinesHandle {
                offsets: data.offsets().into(),
                req_fd: unsafe { OwnedFd::from_raw_fd(data.inner.fd) },
            })
        }
    }
}

fn offsets_to_mask(offsets: &[u32], target_offsets: impl AsRef<[u32]>) -> libc::c_ulong {
    let target_offsets = target_offsets.as_ref();
    let mut mask = 0;
    for (index, &offset) in offsets.iter().enumerate() {
        if target_offsets.contains(&offset) {
            mask |= (1 << index);
        }
    }

    mask
}

fn index_of_offset(offsets: &[u32], target: u32) -> Option<usize> {
    offsets.iter().position(|&o| o == target)
}

pub struct LineValues {
    #[cfg(feature = "v2")]
    inner: ffi::v2::GpioV2LineValues,
    #[cfg(feature = "v1")]
    inner: ffi::v1::GpioHandleData,
    offsets: TinyVec<[u32; 8]>,
}

impl LineValues {
    pub fn value_of_offset(&self, offset: u32) -> Option<u8> {
        let index = index_of_offset(&self.offsets, offset)?;
        self.value_of_index(index)
    }

    fn value_of_index(&self, index: usize) -> Option<u8> {
        #[cfg(feature = "v1")]
        {
            self.inner.values.get(index).copied()
        }
        #[cfg(feature = "v2")]
        {
            if index >= ffi::v2::GPIO_V2_LINES_MAX {
                return None;
            }
            let flag = 1 << index;
            if self.inner.mask & flag != 0 {
                match self.inner.bits & flag {
                    0 => Some(0),
                    _ => Some(1),
                }
            } else {
                None
            }
        }
    }

    pub fn values_iter(&self) -> LineValuesIter<'_> {
        LineValuesIter::new(self)
    }
}

impl Debug for LineValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.values_iter().map(|v| (v.offset, v.value)))
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LineValueItem {
    offset: u32,
    value: u8,
}

#[derive(Debug)]
pub struct LineValuesIter<'a> {
    values: &'a LineValues,
    index: usize,
}

impl<'a> LineValuesIter<'a> {
    pub fn new(values: &'a LineValues) -> Self {
        Self { values, index: 0 }
    }
}

impl Iterator for LineValuesIter<'_> {
    type Item = LineValueItem;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.values.offsets.len() {
            self.index += 1;
            if let Some(value) = self.values.value_of_index(self.index - 1) {
                return Some(LineValueItem {
                    offset: self.values.offsets[self.index - 1],
                    value,
                });
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.values.offsets.len() - self.index))
    }
}

impl Clone for LineValuesIter<'_> {
    fn clone(&self) -> Self {
        Self {
            values: self.values,
            index: 0,
        }
    }
}

pub struct LinesRequestBuilder {
    inner: LinesRequest,
    index: usize,
    edge_dection: bool,
}

impl LinesRequestBuilder {
    pub fn new() -> Self {
        unsafe { std::mem::zeroed() }
    }

    pub fn set_consumer(mut self, consumer: impl AsRef<str>) -> Self {
        #[cfg(feature = "v1")]
        {
            self.inner.inner.consumer_label = consumer.into();
        }
        #[cfg(feature = "v2")]
        {
            self.inner.inner.consumer = consumer.into();
        }

        self
    }

    pub fn set_flags(mut self, flags: LineFlags) -> Self {
        #[cfg(feature = "v1")]
        {
            self.inner.inner.flags = flags.bits();
        }
        #[cfg(feature = "v2")]
        {
            self.inner.inner.config.flags = flags.bits();
        }
        self
    }

    #[cfg(feature = "v2")]
    pub fn set_event_buffer_size(mut self, size: u32) -> Self {
        self.inner.inner.event_buffer_size = size;
        self
    }
}

impl Default for LinesRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}
