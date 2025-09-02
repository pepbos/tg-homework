#![no_std]
#![feature(array_try_map)]

mod crc;
mod decoder;
mod encoder;
mod escape;
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
    InvalidCrc { got: [u8; 2], expected: [u8; 2] },
}
