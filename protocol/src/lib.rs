#![no_std]
#![feature(array_try_map)]

mod crc;
mod decoder;
mod encoder;
mod escape;
mod frame;

pub(crate) use crc::Crc;
pub use decoder::{DecoderState, decode_in_place};
pub use encoder::{encode_in_place, encoded_len};
pub use frame::Header;
pub use escape::*;

pub const SYNC: u8 = 0xFF;

pub const MAX_FRAME_LEN: u8 = 0x80;

pub const MAX_ENCODED_LEN: usize = MAX_FRAME_LEN as usize + frame::HEADER_LEN;

#[derive(Clone, Debug)]
pub enum Error {
    EarlySync,
    LateSync,
    InvalidLen(u8),
    InvalidCrc { got: [u8; 2], expected: [u8; 2] },
    FrameIncomplete(DecoderState),
}
