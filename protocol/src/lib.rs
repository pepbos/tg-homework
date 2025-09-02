#![no_std]

mod crc;
mod magic_byte;
mod encoder;
mod decoder;
mod special_bytes;

pub(crate) use crc::Crc;
pub(crate) use magic_byte::find_magic_byte;
pub(crate) use special_bytes::*;

pub const START: u8 = 0xFF;

pub const MAX_FRAME_LEN: u8 = 0x80;
pub const MIN_FRAME_LEN: u8 = 1;

enum Error {
    EarlyStart,
    LateStart,
    InvalidLength(u8),
    CrcFailed{got: u8, expected: u8},
}
