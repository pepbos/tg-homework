use crate::{*};

#[derive(Copy, Clone)]
pub struct StartByte {_private: ()}

impl StartByte {
    pub fn new_decode(byte: u8) -> Result<Self, Error> {
        if byte != START {
            return Err(Error::LateStart);
        }
        Ok(Self {_private: ()})
    }
}

fn ne_start(byte: u8) -> Result<(), Error> {
    if byte == START {
        Err(Error::EarlyStart)
    } else {
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct MagicByte {
    value: u8,
}

impl MagicByte {
    pub fn new_decode(encoded_byte: u8, crc: &mut Crc) -> Result<Self, Error> {
        ne_start(encoded_byte)?;
        let value = crc.process_encoded_byte(encoded_byte);
        Ok(Self { value })
    }

    pub fn decode_byte(&self, encoded_byte: u8) -> u8 {
        if encoded_byte == self.value {
            START
        } else {
            encoded_byte
        }
    }
}

#[derive(Copy, Clone)]
pub struct LengthByte {
    value: u8,
}

impl LengthByte {
    pub fn new_decode(magic_byte: MagicByte, encoded_byte: u8, crc: &mut Crc) -> Result<Self, Error> {
        ne_start(encoded_byte)?;
        let value = magic_byte.decode_byte(crc.process_encoded_byte(encoded_byte));
        if value > MAX_FRAME_LEN {
            return Err(Error::InvalidLength(value));
        }
        Ok(Self { value })
    }
}

#[derive(Copy, Clone)]
pub struct DataByte {
    value: u8,
}

impl DataByte {
    pub fn new_decode(
        magic_byte: MagicByte,
        encoded_byte: u8,
        crc: &mut Crc,
    ) -> Result<Self, Error> {
        ne_start(encoded_byte)?;
        let value = magic_byte.decode_byte(crc.process_encoded_byte(encoded_byte));
        Ok(Self { value })
    }
}

#[derive(Copy, Clone)]
pub struct CrcByte {
    value: u8,
}

impl CrcByte {
    pub fn new_decode(crc: Crc, magic_byte: MagicByte, encoded_byte: u8) -> Result<Self, Error> {
        let value = magic_byte.decode_byte(encoded_byte);
        let expected = crc.compute();
        if value != expected {
            return Err(Error::CrcFailed {
                got: value,
                expected,
            });
        }
        Ok(Self { value })
    }
}

impl Into<u8> for DataByte {
    fn into(self) -> u8 {
        self.value
    }
}
impl Into<u8> for LengthByte {
    fn into(self) -> u8 {
        self.value
    }
}
impl Into<usize> for LengthByte {
    fn into(self) -> usize {
        self.value as usize
    }
}
