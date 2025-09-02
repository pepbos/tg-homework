use crate::*;

#[derive(Copy, Clone)]
pub struct SyncByte {
    _private: (),
}

impl SyncByte {
    pub fn decode(byte: u8) -> Result<Self, Error> {
        if byte != SYNC {
            return Err(Error::LateSync);
        }
        Ok(Self { _private: () })
    }
}

fn ensure_not_sync(byte: u8) -> Result<(), Error> {
    if byte == SYNC {
        Err(Error::EarlySync)
    } else {
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct EscapeByte {
    value: u8,
}

impl EscapeByte {
    pub fn decode(encoded: u8, crc: &mut Crc) -> Result<Self, Error> {
        ensure_not_sync(encoded)?;
        let value = crc.update(encoded);
        Ok(Self { value })
    }

    fn unescape(&self, encoded_byte: u8) -> u8 {
        if encoded_byte == self.value {
            SYNC
        } else {
            encoded_byte
        }
    }
}

#[derive(Copy, Clone)]
pub struct LenByte {
    value: u8,
}

impl LenByte {
    pub fn decode(encoded: u8, escape: EscapeByte, crc: &mut Crc) -> Result<Self, Error> {
        ensure_not_sync(encoded)?;
        let value = escape.unescape(crc.update(encoded));
        if value > MAX_FRAME_LEN {
            return Err(Error::InvalidLen(value));
        }
        Ok(Self { value })
    }
}

#[derive(Copy, Clone)]
pub struct PayloadByte {
    value: u8,
}

impl PayloadByte {
    pub fn decode(encoded: u8, escape: EscapeByte, crc: &mut Crc) -> Result<Self, Error> {
        ensure_not_sync(encoded)?;
        let value = escape.unescape(crc.update(encoded));
        Ok(Self { value })
    }
}

#[derive(Copy, Clone)]
pub struct CrcByte {
    value: u8,
}

impl CrcByte {
    pub fn decode(encoded: u8, escape: EscapeByte, crc: Crc) -> Result<Self, Error> {
        let value = escape.unescape(encoded);
        let expected = crc.finalize();
        if value != expected {
            return Err(Error::InvalidCrc {
                got: value,
                expected,
            });
        }
        Ok(Self { value })
    }
}

impl Into<u8> for PayloadByte {
    fn into(self) -> u8 {
        self.value
    }
}
impl Into<u8> for LenByte {
    fn into(self) -> u8 {
        self.value
    }
}
impl Into<usize> for LenByte {
    fn into(self) -> usize {
        self.value as usize
    }
}
