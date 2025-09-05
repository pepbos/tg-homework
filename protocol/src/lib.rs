//! Custom UART protocol.
//!
//! This crate contains a protocol for sending data over uart.
//!
//! Data can be send in packages of up to 128 bytes, and one identifier byte, at a time. The data
//! is encoded in a frame, with the following bytes:
//!
//! |       | SYNC | ESC | ID | LEN | PAYLOAD | CRC |
//! | ----- | ---- | --- | -- | --- | ------- | --- |
//! | bytes |  1   |  1  | 1  |  1  |max(128) | 2   |
//!
//! where
//! - [SYNC] signals the start of the frame,
//! - `ESC` is an escape character,
//! - `ID` is an identifier byte (for optional use),
//! - `LEN` is the number of data bytes,
//! - `PAYLOAD` contains the data bytes,
//! - `CRC` is a two byte checksum.
//!
//! When [encoding][encode_in_place] a package to a frame all occurances of the [SYNC] byte in
//! `ID`, `LEN`, `PAYLOAD` and `CRC` are escaped using `ESC`. [Decoding][Decoder::decode_in_place] reverses
//! this process.

#![no_std]
#![feature(array_try_map)]

mod crc;
mod decoder;
mod encoder;
mod escape;
mod header;

pub(crate) use crc::CrcState;
pub(crate) use header::HEADER_LEN;

pub use decoder::Decoder;
pub use encoder::{encode_in_place, encoded_len};
pub use header::Header;
pub use escape::*;

pub const SYNC: u8 = 0xFF;

pub const MAX_FRAME_LEN: u8 = 0x80;

pub const MAX_ENCODED_LEN: usize = MAX_FRAME_LEN as usize + header::HEADER_LEN;

#[derive(Clone, Debug)]
pub enum Error {
    EarlySync,
    LateSync,
    InvalidLen(u8),
    InvalidCrc { got: [u8; 2], expected: [u8; 2] },
}
