#![allow(unused)]

pub mod errors;
mod ffi;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IoctlKind {
    ChipInfo,
    LineInfo,
    LineHandle,
    LineEvent,
    GetLine,
    SetLine,
}
