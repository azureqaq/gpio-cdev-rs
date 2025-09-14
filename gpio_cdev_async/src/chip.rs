//! This module provides an interface to the GPIO chip.
//!
//! # Examples
//! ```rust,no_run
//! # use gpio_cdev_async::Chip;
//! let chip = Chip::new("/dev/gpiochip0").unwrap();
//! let chip_info = chip.get_chipinfo().unwrap();
//!
//! println!("{:?}", chip_info);
//! ```
//!
//! This module is available under both v1 and v2 features.

use std::{
    borrow::Cow,
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

/// Represents a GPIO chip.
#[derive(Debug)]
pub struct Chip {
    pub(crate) file: File,
    path: PathBuf,
}

impl Chip {
    /// Opens a GPIO chip at the specified path.
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use gpio_cdev_async::Chip;
    /// let _chip = Chip::new("/dev/gpiochip0").unwrap();
    /// ```
    ///
    /// # Notes
    /// - This function does not check if the path is a valid GPIO chip.
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

    /// Returns the path of the GPIO chip.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the information of the GPIO chip.
    ///
    /// # Notes
    /// - This function retrieves the chip information from the kernel every time it is called.
    pub fn get_chipinfo(&self) -> Result<ChipInfo> {
        let mut inner: ffi::common::GpioChipInfo = unsafe { std::mem::zeroed() };
        ffi::common::gpio_get_chipinfo_ioctl(self.file.as_raw_fd(), &mut inner)?;
        Ok(ChipInfo { inner })
    }

    /// Get the information of a GPIO line.
    ///
    /// # Arguments
    /// - `offset`: The offset of the GPIO line.
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use gpio_cdev_async::Chip;
    /// let chip = Chip::new("/dev/gpiochip0").unwrap();
    ///
    /// // Get the information of `GPIO6`
    /// let _line_info = chip.get_lineinfo(6).unwrap();
    /// ```
    /// # Notes
    /// - This function retrieves the chip information from the kernel every time it is called.
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

    /// Get a GPIO line handle.
    ///
    /// See [`LineRequest`] for more information.
    pub fn get_line(&self, request: LineRequest) -> Result<LineHandle> {
        request.request(self)
    }

    /// Get a GPIO pin handle.
    ///
    /// See [`PinRequest`] for more information.
    pub fn get_pin(&self, request: PinRequest) -> Result<PinHandle> {
        request.request(self)
    }

    pub fn get_lineinfo_watch(&self, offset: u32) -> Result<LineInfo> {
        #[cfg(feature = "v2")]
        {
            use ffi::v2::GpioV2LineInfo;
            let mut inner: GpioV2LineInfo = unsafe { std::mem::zeroed() };
            inner.offset = offset;
            ffi::v2::gpio_v2_get_lineinfo_watch_ioctl(self.file.as_raw_fd(), &mut inner)?;
            Ok(LineInfo { inner })
        }
        #[cfg(feature = "v1")]
        {
            use ffi::v1::GpioLineInfo;
            let mut inner: GpioLineInfo = unsafe { std::mem::zeroed() };
            inner.line_offset = offset;
            ffi::v1::gpio_get_lineinfo_watch_ioctl(self.file.as_raw_fd(), &mut inner)?;
            Ok(LineInfo { inner })
        }
    }

    pub fn get_lineinfo_unwatch(&self, mut offset: u32) -> Result<()> {
        ffi::common::gpio_get_lineinfo_unwatch_ioctl(self.file.as_raw_fd(), &mut offset)?;
        Ok(())
    }

    // pub fn
}

/// Represents the information of a GPIO chip.
#[repr(transparent)]
pub struct ChipInfo {
    inner: ffi::common::GpioChipInfo,
}

impl ChipInfo {
    /// The name of the GPIO chip.
    pub fn name(&self) -> Cow<'_, str> {
        self.inner.name.to_string_lossy()
    }

    /// The label of the GPIO chip.
    pub fn label(&self) -> Cow<'_, str> {
        self.inner.label.to_string_lossy()
    }

    /// The number of GPIO lines on the chip.
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
