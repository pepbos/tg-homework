#![no_std]

mod magic_byte;

pub(crate) use magic_byte::find_magic_byte;

pub const MAX_FRAME_LEN: u8 = 128;
pub const MIN_FRAME_LEN: u8 = 1;
