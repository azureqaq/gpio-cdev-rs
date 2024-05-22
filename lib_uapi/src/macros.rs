macro_rules! wrap_ioctl {
    ($ioctl_macro:ident!($name:ident, $ioty:expr, $nr:expr, $ty:ty), $ioctl_error_ty:expr) => {
        mod $name {
            nix::$ioctl_macro!($name, $ioty, $nr, $ty);
        }

        pub(crate) fn $name(fd: libc::c_int, data: &mut $ty) -> $crate::error::Result<libc::c_int> {
            unsafe {
                $name::$name(fd, data).map_err(|e| $crate::error::ioctl_error($ioctl_error_ty, e))
            }
        }
    };
}

macro_rules! const_assert {
    ($($tt:tt)*) => {
        const _: () = assert!($($tt)*);
    };
}

pub(crate) use const_assert;
pub(crate) use wrap_ioctl;
