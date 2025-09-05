use crate::*;

#[derive(Clone, Debug)]
pub struct Decoder {
    state: DecoderState,
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            state: DecoderState::AwaitingSync,
        }
    }

    pub fn decode_in_place(
        &mut self,
        encoded: u8,
        out: &mut [u8],
    ) -> Result<Option<Header>, Error> {
        self.state.decode_byte_in_place(encoded, out)?;
        match self.state {
            DecoderState::FrameComplete(header) => Ok(Some(header)),
            _ => Ok(None),
        }
    }
}

/// State machine for decoding a received frame.
#[derive(Clone, Debug)]
pub(crate) enum DecoderState {
    AwaitingSync,
    AwaitingEscape {
        sync: u8,
    },
    AwaitingId {
        sync: u8,
        escape: Escape,
        crc: CrcState,
    },
    AwaitingLen {
        sync: u8,
        escape: Escape,
        id: u8,
        crc: CrcState,
    },
    ReadingPayload {
        sync: u8,
        escape: Escape,
        id: u8,
        len: u8,
        index: usize,
        crc: CrcState,
    },
    AwaitingCrc {
        sync: u8,
        escape: Escape,
        id: u8,
        len: u8,
        crc: CrcState,
        byte_buffer: Option<u8>,
    },
    FrameComplete(Header),
}

impl Default for DecoderState {
    fn default() -> Self {
        Self::AwaitingSync
    }
}

fn verify_sync(encoded: u8) -> Result<u8, Error> {
    if encoded != SYNC {
        return Err(Error::LateSync);
    }
    Ok(encoded)
}

fn unescape_and_update_crc(encoded: u8, escape: &Escape, crc: &mut CrcState) -> Result<u8, Error> {
    Ok(crc.digest_single(escape.unescape(encoded)?))
}

fn verify_len(len: u8) -> Result<u8, Error> {
    if len > MAX_FRAME_LEN {
        Err(Error::InvalidLen(len))
    } else {
        Ok(len)
    }
}

fn verify_checksum(checksum: [u8; 2], crc: CrcState) -> Result<[u8; 2], Error> {
    let expected = crc.finalize();
    let got = checksum;
    if got
        .iter()
        .zip(expected.iter())
        .all(|(left, right)| left == right)
    {
        Ok(checksum)
    } else {
        Err(Error::InvalidCrc { got, expected })
    }
}

impl DecoderState {
    pub fn decode_byte_in_place(
        &mut self,
        encoded_byte: u8,
        out: &mut [u8],
    ) -> Result<Option<&Header>, Error> {
        *self = core::mem::take(self).calc_updated(encoded_byte, out)?;
        match self {
            DecoderState::FrameComplete(header) => Ok(Some(header)),
            _ => Ok(None),
        }
    }

    fn calc_updated(self, encoded: u8, out: &mut [u8]) -> Result<DecoderState, Error> {
        match self {
            DecoderState::AwaitingSync => Ok(DecoderState::AwaitingEscape {
                sync: verify_sync(encoded)?,
            }),
            DecoderState::AwaitingEscape { sync } => {
                let escape = Escape::try_from_raw(encoded)?;
                Ok(DecoderState::AwaitingId {
                    sync,
                    escape,
                    crc: CrcState::new(),
                })
            }
            DecoderState::AwaitingId {
                sync,
                escape,
                mut crc,
            } => {
                let id = unescape_and_update_crc(encoded, &escape, &mut crc)?;
                Ok(DecoderState::AwaitingLen {
                    sync,
                    escape,
                    id,
                    crc,
                })
            }
            DecoderState::AwaitingLen {
                sync,
                escape,
                id,
                mut crc,
            } => {
                let len =
                    unescape_and_update_crc(encoded, &escape, &mut crc).and_then(verify_len)?;
                Ok(DecoderState::ReadingPayload {
                    sync,
                    escape,
                    crc,
                    len,
                    index: 0,
                    id,
                })
            }
            DecoderState::ReadingPayload {
                sync,
                escape,
                id,
                len,
                index,
                mut crc,
            } => {
                out[index] = unescape_and_update_crc(encoded, &escape, &mut crc)?;
                Ok(if index + 1 >= len.into() {
                    DecoderState::AwaitingCrc {
                        sync,
                        escape,
                        len,
                        byte_buffer: None,
                        crc,
                        id,
                    }
                } else {
                    DecoderState::ReadingPayload {
                        sync,
                        escape,
                        len,
                        index: index + 1,
                        crc,
                        id,
                    }
                })
            }
            DecoderState::AwaitingCrc {
                sync,
                escape,
                id,
                len,
                crc,
                byte_buffer,
            } => {
                if let Some(first) = byte_buffer {
                    let second = encoded;
                    let checksum = [first, second]
                        .try_map(|b| escape.unescape(b))
                        .and_then(|checksum| verify_checksum(checksum, crc))?;
                    Ok(DecoderState::FrameComplete(Header {
                        sync,
                        escape: escape.into(),
                        id,
                        len,
                        crc: checksum,
                    }))
                } else {
                    Ok(DecoderState::AwaitingCrc {
                        sync,
                        escape,
                        id,
                        len,
                        byte_buffer: Some(encoded),
                        crc,
                    })
                }
            }
            DecoderState::FrameComplete { .. } => {
                DecoderState::AwaitingSync.calc_updated(encoded, out)
            }
        }
    }
}
