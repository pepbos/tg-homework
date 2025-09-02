#![no_std]

mod crc;
mod escape;
mod encoder;
mod decoder;
mod special_bytes;

pub(crate) use crc::Crc;
pub(crate) use escape::find_escape;
pub(crate) use special_bytes::*;

pub const SYNC: u8 = 0xFF;

pub const MAX_FRAME_LEN: u8 = 0x80;
pub const MIN_FRAME_LEN: u8 = 1;

enum Error {
    EarlySync,
    LateSync,
    InvalidLen(u8),
    InvalidCrc{got: u8, expected: u8},
}
