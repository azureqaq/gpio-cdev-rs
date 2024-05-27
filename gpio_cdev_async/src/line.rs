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
    pub fn attrs(&self) -> Vec<LineAttribute> {
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

pub struct LineHandle {
    offsets: Vec<u32>,
    req_fd: OwnedFd,
}

impl Debug for LineHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinesHandle")
            .field("offsets", &self.offsets.as_slice())
            .field("req_fd", &self.req_fd)
            .finish()
    }
}

impl LineHandle {
    pub fn offsets(&self) -> &[u32] {
        &self.offsets
    }

    pub fn get_values(&self) -> Result<LineValue> {
        #[cfg(feature = "v1")]
        {
            let mut data: ffi::v1::GpioHandleData = unsafe { std::mem::zeroed() };
            ffi::v1::gpiohandle_get_line_values_ioctl(self.req_fd.as_raw_fd(), &mut data)?;
            Ok(LineValue {
                inner: data,
                offsets: self.offsets.clone(),
            })
        }
        #[cfg(feature = "v2")]
        {
            let mut mask = 0;
            for index in 0..self.offsets.len() {
                mask |= 1 << index;
            }
            self.get_values_by_mask(mask)
        }
    }

    #[cfg(feature = "v2")]
    pub fn get_values_by_mask(&self, mask: libc::c_ulong) -> Result<LineValue> {
        let mut data: ffi::v2::GpioV2LineValues = unsafe { std::mem::zeroed() };
        data.mask = mask;
        ffi::v2::gpio_v2_line_get_values_ioctl(self.req_fd.as_raw_fd(), &mut data)?;
        Ok(LineValue {
            inner: data,
            offsets: self.offsets.clone(),
        })
    }

    #[cfg(feature = "v2")]
    pub fn get_values_by_offsets(&self, offsets: impl AsRef<[u32]>) -> Result<LineValue> {
        let mask = offsets_to_mask(self.offsets(), offsets);
        self.get_values_by_mask(mask)
    }

    #[cfg(feature = "v2")]
    fn set_values_by_mask(&self, mask: libc::c_ulong, bits: libc::c_ulong) -> Result<()> {
        let mut data: ffi::v2::GpioV2LineValues = unsafe { std::mem::zeroed() };

        data.mask = mask;
        data.bits = bits;
        ffi::v2::gpio_v2_line_set_values_ioctl(self.req_fd.as_raw_fd(), &mut data)?;
        Ok(())
    }

    #[cfg(feature = "v2")]
    pub fn set_values<I, T>(&self, offsets: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        T: Into<LineValueItem>,
    {
        let mut mask = 0;
        let mut bits = 0;
        for LineValueItem { offset, value } in offsets.into_iter().map(Into::into) {
            if let Some(index) = index_of_offset(&self.offsets, offset) {
                let flag = 1 << index;
                mask |= flag;
                if value != 0 {
                    bits |= flag;
                }
            }
        }
        self.set_values_by_mask(mask, bits)
    }

    #[cfg(feature = "v1")]
    pub fn set_values<I>(&self, offsets: I) -> Result<()>
    where
        I: IntoIterator<Item = u32>,
    {
        let mut data: ffi::v1::GpioHandleData = unsafe { std::mem::zeroed() };
        for offset in offsets.into_iter() {
            if let Some(index) = index_of_offset(&self.offsets, offset) {
                data.values[index] = 1;
            }
        }
        ffi::v1::gpiohandle_set_line_values_ioctl(self.req_fd.as_raw_fd(), &mut data)?;
        Ok(())
    }
}

#[repr(transparent)]
pub struct LineRequest {
    #[cfg(feature = "v1")]
    inner: ffi::v1::GpioHandleRequest,
    #[cfg(feature = "v2")]
    inner: ffi::v2::GpioV2LineRequest,
}

impl LineRequest {
    pub fn builder() -> LineRequestBuilder {
        LineRequestBuilder::new()
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
    // FIXME: Ambiguous return value
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

impl LineRequest {
    pub fn request(self, chip: &Chip) -> Result<LineHandle> {
        #[cfg(feature = "v2")]
        {
            let mut data = self;
            ffi::v2::gpio_v2_get_line_ioctl(chip.file.as_raw_fd(), &mut data.inner)?;
            Ok(LineHandle {
                offsets: data.offsets().into(),
                req_fd: unsafe { OwnedFd::from_raw_fd(data.inner.fd) },
            })
        }
        #[cfg(feature = "v1")]
        {
            let mut data = self;
            ffi::v1::gpio_get_linehandle_ioctl(chip.file.as_raw_fd(), &mut data.inner)?;
            Ok(LineHandle {
                offsets: data.offsets().into(),
                req_fd: unsafe { OwnedFd::from_raw_fd(data.inner.fd) },
            })
        }
    }
}

impl Debug for LineRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = f.debug_struct("LinesRequest");
        res.field("offsets", &self.offsets());
        res.field("consumer", &self.consumer());
        res.field("flags", &self.flags());
        #[cfg(feature = "v2")]
        res.field("attrs", &self.attrs());
        res.finish()
    }
}

#[cfg(feature = "v2")]
fn offsets_to_mask(offsets: &[u32], target_offsets: impl AsRef<[u32]>) -> libc::c_ulong {
    let target_offsets = target_offsets.as_ref();
    let mut mask = 0;
    for (index, &offset) in offsets.iter().enumerate() {
        if target_offsets.contains(&offset) {
            mask |= 1 << index;
        }
    }

    mask
}

fn index_of_offset(offsets: &[u32], target: u32) -> Option<usize> {
    offsets.iter().position(|&o| o == target)
}

pub struct LineValue {
    #[cfg(feature = "v2")]
    inner: ffi::v2::GpioV2LineValues,
    #[cfg(feature = "v1")]
    inner: ffi::v1::GpioHandleData,
    offsets: Vec<u32>,
}

impl LineValue {
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

    pub fn values_iter(&self) -> LineValueIter<'_> {
        LineValueIter::new(self)
    }
}

impl Debug for LineValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.values_iter().map(|v| (v.offset, v.value)))
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LineValueItem {
    pub offset: u32,
    pub value: u8,
}

impl From<(u32, u8)> for LineValueItem {
    fn from((offset, value): (u32, u8)) -> Self {
        Self { offset, value }
    }
}

impl From<(u32, bool)> for LineValueItem {
    fn from((offset, value): (u32, bool)) -> Self {
        Self {
            offset,
            value: if value { 1 } else { 0 },
        }
    }
}

impl From<u32> for LineValueItem {
    fn from(offset: u32) -> Self {
        Self { offset, value: 1 }
    }
}

#[derive(Debug)]
pub struct LineValueIter<'a> {
    values: &'a LineValue,
    index: usize,
}

impl<'a> LineValueIter<'a> {
    pub fn new(values: &'a LineValue) -> Self {
        Self { values, index: 0 }
    }
}

impl Iterator for LineValueIter<'_> {
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

impl Clone for LineValueIter<'_> {
    fn clone(&self) -> Self {
        Self {
            values: self.values,
            index: 0,
        }
    }
}

pub struct LineRequestBuilder {
    inner: LineRequest,
}

impl LineRequestBuilder {
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

    pub fn set_flags(mut self, flags: HandleFlags) -> Self {
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

    pub fn set_offsets<I, T>(mut self, configs: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<OffsetConfig>,
    {
        #[cfg(feature = "v2")]
        {
            // also as line index
            let mut lines_num = 0;
            // also as attr index
            let mut attrs_num = 0;

            'outer: for config in configs
                .into_iter()
                .map(Into::<OffsetConfig>::into)
                .take(self.inner.inner.offsets.len())
            {
                // set offset
                self.inner.inner.offsets[lines_num as usize] = config.offset;
                // set attr
                for attr in config.line_attr {
                    let attr_config = &mut self.inner.inner.config.attrs[attrs_num as usize];
                    attr_config.mask = 1 << lines_num;

                    attr_config.attr = attr.into_line_attribute(lines_num);

                    attrs_num += 1;
                    if attrs_num as usize >= self.inner.inner.config.attrs.len() {
                        lines_num += 1;
                        break 'outer;
                    }
                }

                lines_num += 1;
            }

            self.inner.inner.num_lines = lines_num;
            self.inner.inner.config.num_attrs = attrs_num;
        }

        #[cfg(feature = "v1")]
        {
            let mut lines_num = 0;

            for config in configs
                .into_iter()
                .map(Into::<OffsetConfig>::into)
                .take(self.inner.inner.lineoffsets.len())
            {
                self.inner.inner.lineoffsets[lines_num as usize] = config.offset;
                self.inner.inner.default_values[lines_num as usize] =
                    config.default_value.unwrap_or_default();
                lines_num += 1;
            }

            self.inner.inner.lines = lines_num;
        }

        self
    }

    #[cfg(feature = "v2")]
    pub fn set_event_buffer_size(mut self, size: u32) -> Self {
        self.inner.inner.event_buffer_size = size;
        self
    }

    pub fn build(self) -> Result<LineRequest> {
        // TODO: check config
        Ok(self.inner)
    }
}

impl Default for LineRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct OffsetConfig {
    offset: u32,
    #[cfg(feature = "v1")]
    default_value: Option<u8>,
    #[cfg(feature = "v2")]
    line_attr: Vec<OffsetAttribute>,
}

#[cfg(feature = "v2")]
impl<T> From<(u32, T)> for OffsetConfig
where
    T: Into<Vec<OffsetAttribute>>,
{
    fn from(value: (u32, T)) -> Self {
        Self {
            offset: value.0,
            line_attr: value.1.into(),
        }
    }
}

#[cfg(feature = "v1")]
impl From<(u32, u8)> for OffsetConfig {
    fn from((offset, default_value): (u32, u8)) -> Self {
        Self {
            offset,
            default_value: Some(default_value),
        }
    }
}

impl From<u32> for OffsetConfig {
    fn from(value: u32) -> Self {
        #[cfg(feature = "v2")]
        {
            Self {
                offset: value,
                line_attr: Vec::default(),
            }
        }
        #[cfg(feature = "v1")]
        {
            Self {
                offset: value,
                default_value: None,
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg(feature = "v2")]
pub enum OffsetAttribute {
    Flags(LineFlags),
    Value(u8),
    DebouncePeriodUs(u32),
}

#[cfg(feature = "v2")]
impl OffsetAttribute {
    fn into_line_attribute(self, index: u32) -> ffi::v2::GpioV2LineAttribute {
        match self {
            Self::Value(v) => ffi::v2::GpioV2LineAttribute {
                id: ffi::v2::GpioV2LineAttrId::OutputValues as u32,
                padding: ffi::common::Padding([0]),
                u: ffi::v2::Union {
                    values: if v == 0 { 0 } else { 1 << index },
                },
            },
            Self::Flags(flags) => ffi::v2::GpioV2LineAttribute {
                id: ffi::v2::GpioV2LineAttrId::Flags as u32,
                padding: ffi::common::Padding([0]),
                u: ffi::v2::Union {
                    flags: flags.bits(),
                },
            },
            Self::DebouncePeriodUs(us) => ffi::v2::GpioV2LineAttribute {
                id: ffi::v2::GpioV2LineAttrId::Debounce as u32,
                padding: ffi::common::Padding([0]),
                u: ffi::v2::Union {
                    debounce_period_us: us,
                },
            },
        }
    }
}

#[cfg(feature = "v2")]
impl From<LineFlags> for OffsetAttribute {
    fn from(value: LineFlags) -> Self {
        Self::Flags(value)
    }
}

#[cfg(feature = "v2")]
impl From<bool> for OffsetAttribute {
    fn from(value: bool) -> Self {
        Self::Value(if value { 1 } else { 0 })
    }
}

#[cfg(feature = "v2")]
impl From<u32> for OffsetAttribute {
    fn from(value: u32) -> Self {
        Self::DebouncePeriodUs(value)
    }
}

#[derive(Debug)]
pub struct OffsetHandle {
    line_handle: LineHandle,
}

impl OffsetHandle {
    pub fn offset(&self) -> u32 {
        self.line_handle.offsets[0]
    }

    pub fn get_value(&self) -> Result<u8> {
        let values = self.line_handle.get_values()?;
        Ok(values.value_of_index(0).unwrap())
    }

    pub fn set_value(&self, value: u8) -> Result<()> {
        #[cfg(feature = "v2")]
        {
            self.line_handle.set_values([(self.offset(), value)])
        }
        #[cfg(feature = "v1")]
        {
            if value != 0 {
                self.line_handle.set_values([self.offset()])
            } else {
                self.line_handle.set_values([])
            }
        }
    }
}

#[derive(Debug)]
pub struct OffsetRequest {
    line_request: LineRequest,
}

impl OffsetRequest {
    pub fn new(
        offset: u32,
        flags: HandleFlags,
        default_value: u8,
        consumer: impl AsRef<str>,
    ) -> Self {
        let line_request_builder = LineRequestBuilder::new()
            .set_flags(flags)
            .set_consumer(consumer);

        #[cfg(feature = "v2")]
        let line_request_builder =
            line_request_builder.set_offsets([(offset, [OffsetAttribute::Value(default_value)])]);

        #[cfg(feature = "v1")]
        let line_request_builder = line_request_builder.set_offsets([(offset, default_value)]);

        Self {
            line_request: line_request_builder.build().unwrap(),
        }
    }

    pub fn offset(&self) -> u32 {
        self.line_request.offsets()[0]
    }

    pub fn consumer(&self) -> Cow<'_, str> {
        self.line_request.consumer()
    }

    pub fn flags(&self) -> HandleFlags {
        self.line_request.flags()
    }

    pub fn default_value(&self) -> Option<u8> {
        self.line_request.default_value_of_offset(0)
    }
}

impl OffsetRequest {
    pub fn request(self, chip: &Chip) -> Result<OffsetHandle> {
        debug_assert_eq!(self.line_request.offsets().len(), 1);
        // TODO: check config?
        self.line_request
            .request(chip)
            .map(|line_handle| OffsetHandle { line_handle })
    }
}
