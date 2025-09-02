use crate::*;

#[derive(Clone, Debug)]
pub enum DecoderState {
    AwaitingSync,
    AwaitingEscape {
        start: SyncByte,
        crc: Crc,
    },
    AwaitingLen {
        start: SyncByte,
        escape: EscapeByte,
        crc: Crc,
    },
    ReadingPayload {
        start: SyncByte,
        escape: EscapeByte,
        len: LenByte,
        index: usize,
        crc: Crc,
    },
    AwaitingCrc {
        start: SyncByte,
        escape: EscapeByte,
        len: LenByte,
        crc: Crc,
        byte_buffer: Option<u8>,
    },
    FrameComplete {
        start: SyncByte,
        escape: EscapeByte,
        len: LenByte,
        crc: CrcByte,
    },
}

impl Default for DecoderState {
    fn default() -> Self {
        Self::AwaitingSync
    }
}

pub fn decode_in_place(encoded: &[u8], out: &mut [u8]) -> Result<usize, Error> {
    let mut decoder = DecoderState::default();
    for &b in encoded.iter() {
        decoder.decode_byte_in_place(b, out)?;
    }
    match decoder {
        DecoderState::FrameComplete { len, .. } => Ok(len.into()),
        _ => Err(Error::FrameIncomplete(decoder)),
    }
}

impl DecoderState {
    pub fn decode_byte_in_place(
        &mut self,
        encoded_byte: u8,
        out: &mut [u8],
    ) -> Result<Option<usize>, Error> {
        *self = core::mem::take(self).calc_updated(encoded_byte, out)?;
        match self {
            DecoderState::FrameComplete { len, .. } => Ok(Some((*len).into())),
            _ => Ok(None),
        }
    }

    fn calc_updated(self, encoded: u8, out: &mut [u8]) -> Result<DecoderState, Error> {
        match self {
            DecoderState::AwaitingSync => Ok(DecoderState::AwaitingEscape {
                start: SyncByte::decode(encoded)?,
                crc: Crc::default(),
            }),
            DecoderState::AwaitingEscape { mut crc, start } => {
                let escape = EscapeByte::decode(encoded, &mut crc)?;
                Ok(DecoderState::AwaitingLen { start, escape, crc })
            }
            DecoderState::AwaitingLen {
                start,
                escape,
                mut crc,
            } => {
                let len = LenByte::decode(encoded, escape, &mut crc)?;
                Ok(DecoderState::ReadingPayload {
                    start,
                    escape,
                    crc,
                    len,
                    index: 0,
                })
            }
            DecoderState::ReadingPayload {
                start,
                escape,
                len,
                index,
                mut crc,
            } => {
                out[index] = PayloadByte::decode(encoded, escape, &mut crc)?.into();
                Ok(if index + 1 >= len.into() {
                    DecoderState::AwaitingCrc {
                        start,
                        escape,
                        len,
                        byte_buffer: None,
                        crc,
                    }
                } else {
                    DecoderState::ReadingPayload {
                        start,
                        escape,
                        len,
                        index: index + 1,
                        crc,
                    }
                })
            }
            DecoderState::AwaitingCrc {
                start,
                escape,
                len,
                crc,
                byte_buffer,
            } => Ok(if let Some(first) = byte_buffer {
                let second = encoded;
                DecoderState::FrameComplete {
                    start,
                    escape,
                    len,
                    crc: CrcByte::decode([first, second], escape, crc)?,
                }
            } else {
                DecoderState::AwaitingCrc {
                    start,
                    escape,
                    len,
                    byte_buffer: Some(encoded),
                    crc,
                }
            }),
            DecoderState::FrameComplete { .. } => Ok(DecoderState::AwaitingEscape {
                start: SyncByte::decode(encoded)?,
                crc: Crc::default(),
            }),
        }
    }
}
