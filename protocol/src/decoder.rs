use crate::*;

enum DecoderState {
    AwaitingSync,
    AwaitingEscape {
        start: StartByte,
        crc: Crc,
    },
    AwaitingLen {
        start: StartByte,
        magic_byte: MagicByte,
        crc: Crc,
    },
    ReadingPayload {
        start: StartByte,
        magic_byte: MagicByte,
        len: LengthByte,
        index: u8,
        crc: Crc,
    },
    AwaitingCrc {
        start: StartByte,
        magic_byte: MagicByte,
        len: LengthByte,
        crc: Crc,
    },
    FrameComplete {
        start: StartByte,
        magic_byte: MagicByte,
        len: LengthByte,
        crc: CrcByte,
    },
}

impl Default for DecoderState {
    fn default() -> Self {
        Self::AwaitingSync
    }
}

impl DecoderState {
    pub fn decode_byte_in_place(&mut self, encoded_byte: u8, out: &mut [u8]) -> Result<Option<usize>, Error> {
        *self =
            core::mem::take(self).calc_updated(encoded_byte, out)?;
        match self {
            DecoderState::FrameComplete { len, ..} => Ok(Some((*len).into())),
            _ => Ok(None),
        }
    }

    fn calc_updated(self, encoded_byte: u8, out: &mut [u8]) -> Result<DecoderState, Error> {
        match self {
            DecoderState::AwaitingSync => Ok(DecoderState::AwaitingEscape {
                start: StartByte::new_decode(encoded_byte)?,
                crc: Crc::new(),
            }),
            DecoderState::AwaitingEscape { mut crc, start } => {
                let magic_byte = MagicByte::new_decode(encoded_byte, &mut crc)?;
                Ok(DecoderState::AwaitingLen {
                    start,
                    magic_byte,
                    crc,
                })
            }
            DecoderState::AwaitingLen {
                start,
                magic_byte,
                mut crc,
            } => {
                let len = LengthByte::new_decode(magic_byte, encoded_byte, &mut crc)?;
                Ok(DecoderState::ReadingPayload {
                    start,
                    magic_byte,
                    crc,
                    len,
                    index: 0,
                })
            }
            DecoderState::ReadingPayload {
                start,
                magic_byte,
                len,
                mut index,
                mut crc,
            } => {
                out[index as usize] =
                    DataByte::new_decode(magic_byte, encoded_byte, &mut crc)?.into();
                index += 1;
                Ok(if index >= len.into() {
                    DecoderState::AwaitingCrc {
                        start,
                        magic_byte,
                        len,
                        crc,
                    }
                } else {
                    self
                })
            }
            DecoderState::AwaitingCrc {
                start,
                magic_byte,
                len,
                crc,
            } => Ok(DecoderState::FrameComplete {
                start,
                magic_byte,
                len,
                crc: CrcByte::new_decode(crc, magic_byte, encoded_byte)?,
            }),
            DecoderState::FrameComplete { .. } => Ok(DecoderState::AwaitingEscape {
                start: StartByte::new_decode(encoded_byte)?,
                crc: Crc::new(),
            }),
        }
    }
}
