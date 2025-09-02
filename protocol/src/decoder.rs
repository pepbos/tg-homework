use crate::*;

enum Header {
    Start,
    MagicByte {
        start: StartByte,
        crc: Crc,
    },
    Length {
        start: StartByte,
        magic_byte: MagicByte,
        crc: Crc,
    },
    Data {
        start: StartByte,
        magic_byte: MagicByte,
        len: LengthByte,
        index: u8,
        crc: Crc,
    },
    Crc {
        start: StartByte,
        magic_byte: MagicByte,
        len: LengthByte,
        crc: Crc,
    },
    Complete {
        start: StartByte,
        magic_byte: MagicByte,
        len: LengthByte,
        crc: CrcByte,
    },
}

impl Default for Header {
    fn default() -> Self {
        Self::Start
    }
}

impl Header {
    pub fn decode_byte_in_place(&mut self, encoded_byte: u8, out: &mut [u8]) -> Result<Option<usize>, Error> {
        *self =
            core::mem::take(self).calc_updated(encoded_byte, out)?;
        match self {
            Header::Complete { len, ..} => Ok(Some((*len).into())),
            _ => Ok(None),
        }
    }

    fn calc_updated(self, encoded_byte: u8, out: &mut [u8]) -> Result<Header, Error> {
        match self {
            Header::Start => Ok(Header::MagicByte {
                start: StartByte::new_decode(encoded_byte)?,
                crc: Crc::new(),
            }),
            Header::MagicByte { mut crc, start } => {
                let magic_byte = MagicByte::new_decode(encoded_byte, &mut crc)?;
                Ok(Header::Length {
                    start,
                    magic_byte,
                    crc,
                })
            }
            Header::Length {
                start,
                magic_byte,
                mut crc,
            } => {
                let len = LengthByte::new_decode(magic_byte, encoded_byte, &mut crc)?;
                Ok(Header::Data {
                    start,
                    magic_byte,
                    crc,
                    len,
                    index: 0,
                })
            }
            Header::Data {
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
                    Header::Crc {
                        start,
                        magic_byte,
                        len,
                        crc,
                    }
                } else {
                    self
                })
            }
            Header::Crc {
                start,
                magic_byte,
                len,
                crc,
            } => Ok(Header::Complete {
                start,
                magic_byte,
                len,
                crc: CrcByte::new_decode(crc, magic_byte, encoded_byte)?,
            }),
            Header::Complete { .. } => Ok(Header::MagicByte {
                start: StartByte::new_decode(encoded_byte)?,
                crc: Crc::new(),
            }),
        }
    }
}
