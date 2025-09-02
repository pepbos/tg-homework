use crate::*;

enum DecoderState {
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
        index: u8,
        crc: Crc,
    },
    AwaitingCrc {
        start: SyncByte,
        escape: EscapeByte,
        len: LenByte,
        crc: Crc,
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
                mut index,
                mut crc,
            } => {
                out[index as usize] = PayloadByte::decode(encoded, escape, &mut crc)?.into();
                index += 1;
                Ok(if index >= len.into() {
                    DecoderState::AwaitingCrc {
                        start,
                        escape,
                        len,
                        crc,
                    }
                } else {
                    self
                })
            }
            DecoderState::AwaitingCrc {
                start,
                escape,
                len,
                crc,
            } => Ok(DecoderState::FrameComplete {
                start,
                escape,
                len,
                crc: CrcByte::decode(encoded, escape, crc)?,
            }),
            DecoderState::FrameComplete { .. } => Ok(DecoderState::AwaitingEscape {
                start: SyncByte::decode(encoded)?,
                crc: Crc::default(),
            }),
        }
    }
}
