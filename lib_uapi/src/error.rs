#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Ioctl to {:?} failed: {}", .kind, .source)]
    Ioctl { kind: IoctlKind, source: nix::Error },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoctlKind {
    GetChipInfo,
    GetLineInfo,
    GetLine,
    SetLineConfig,
    GetValues,
    SetValues,
    GetLineEvent,
}

pub(crate) fn ioctl_error(kind: IoctlKind, source: nix::Error) -> Error {
    Error::Ioctl { kind, source }
}

pub type Result<T> = std::result::Result<T, Error>;
