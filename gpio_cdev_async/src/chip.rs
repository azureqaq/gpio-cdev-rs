use std::{
    borrow::Cow,
    ffi::CStr,
    fmt::Debug,
    fs::File,
    os::fd::AsRawFd,
    path::{Path, PathBuf},
};

use crate::{
    ffi,
    line::{LineHandle, LineInfo, LineRequest, PinHandle, PinRequest},
    Result,
};

#[derive(Debug)]
pub struct Chip {
    pub(crate) file: File,
    path: PathBuf,
}

impl Chip {
    pub fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path.as_ref())?;
        Ok(Self {
            file,
            path: path.as_ref().to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn get_chipinfo(&self) -> Result<ChipInfo> {
        let mut inner: ffi::common::GpioChipInfo = unsafe { std::mem::zeroed() };
        ffi::common::gpio_get_chipinfo_ioctl(self.file.as_raw_fd(), &mut inner)?;
        Ok(ChipInfo { inner })
    }

    pub fn get_lineinfo(&self, offset: u32) -> Result<LineInfo> {
        #[cfg(feature = "v2")]
        {
            use ffi::v2::GpioV2LineInfo;
            let mut inner: GpioV2LineInfo = unsafe { std::mem::zeroed() };
            inner.offset = offset;
            ffi::v2::gpio_v2_get_lineinfo_ioctl(self.file.as_raw_fd(), &mut inner)?;
            Ok(LineInfo { inner })
        }
        #[cfg(feature = "v1")]
        {
            use ffi::v1::GpioLineInfo;
            let mut inner: GpioLineInfo = unsafe { std::mem::zeroed() };
            inner.line_offset = offset;
            ffi::v1::gpio_get_lineinfo_ioctl(self.file.as_raw_fd(), &mut inner)?;
            Ok(LineInfo { inner })
        }
    }

    pub fn get_line(&self, request: LineRequest) -> Result<LineHandle> {
        request.request(self)
    }

    pub fn get_pin(&self, request: PinRequest) -> Result<PinHandle> {
        request.request(self)
    }
}

pub struct ChipInfo {
    inner: ffi::common::GpioChipInfo,
}

impl ChipInfo {
    pub fn name(&self) -> Cow<'_, str> {
        self.inner.name.to_string_lossy()
    }

    pub fn label(&self) -> Cow<'_, str> {
        self.inner.label.to_string_lossy()
    }

    pub fn lines(&self) -> u32 {
        self.inner.lines
    }
}

impl Debug for ChipInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChipInfo")
            .field("name", &self.name())
            .field("label", &self.label())
            .field("lines", &self.lines())
            .finish()
    }
}
