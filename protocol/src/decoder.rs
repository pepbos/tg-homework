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

fn check_length_byte(byte: u8) -> Result<(), Error> {
    if byte > MAX_FRAME_LEN {
        return Err(Error::InvalidLength(byte));
    }
    Ok(())
}

fn check_early_start(byte: u8) -> Result<(), Error> {
    todo!()
}

impl Header {
    fn reset_on_err(&mut self, result: Result<(), Error>) -> Result<(), Error> {
        if result.is_err() {
            *self = Header::Start;
        }
        result
    }

    fn decode_in_place(&mut self, encoded_byte: u8, out: &mut [u8]) -> Result<(), Error> {
        match *self {
            Header::Start => {
                *self = Header::MagicByte {
                    start: StartByte::new_decode(encoded_byte)?,
                    crc: Crc::new(),
                };
            }
            Header::MagicByte { crc, start } => {
                let mut crc = crc;
                // TODO Error if START
                let magic_byte = MagicByte::new_decode(encoded_byte, &mut crc)?;
                *self = Header::Length {
                    start,
                    magic_byte,
                    crc,
                }
            }
            Header::Length {
                magic_byte,
                crc,
                start,
            } => {
                let mut crc = crc;
                let len = LengthByte::new_decode(magic_byte, encoded_byte, &mut crc)?;
                *self = Header::Data {
                    start,
                    magic_byte,
                    crc,
                    len,
                    index: 0,
                }
            }
            Header::Data {
                magic_byte,
                crc,
                len,
                index,
                start,
            } => {
                let mut crc = crc;
                let mut index = index;
                out[index as usize] =
                    DataByte::new_decode(magic_byte, encoded_byte, &mut crc)?.into();
                index += 1;
                if index >= len.into() {
                    *self = Header::Crc {
                        start,
                        magic_byte,
                        len,
                        crc,
                    }
                }
            }
            Header::Crc {
                magic_byte,
                crc,
                start,
                len,
            } => {
                *self = Header::Complete {
                    start,
                    magic_byte,
                    len,
                    crc: CrcByte::new_decode(crc, magic_byte, encoded_byte)?,
                }
            }
            Header::Complete {..} => {
                *self = Header::MagicByte {
                    start: StartByte::new_decode(encoded_byte)?,
                    crc: Crc::new(),
                };
            }
        }
        Ok(())
    }
}
