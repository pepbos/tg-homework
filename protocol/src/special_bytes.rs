use crate::*;

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
pub struct CrcByte {
    _private: (),
}

impl CrcByte {
    pub fn decode(encoded: [u8; 2], escape: EscapeByte, crc: Crc) -> Result<Self, Error> {
        let value = encoded.try_map(|b| escape.unescape(b))?;
        let expected = crc.finalize();
        if !value
            .iter()
            .zip(expected.iter())
            .all(|(got, exp)| got == exp)
        {
            return Err(Error::InvalidCrc {
                got: value,
                expected,
            });
        }
        Ok(Self { _private: () })
    }
}

impl From<PayloadByte> for u8 {
    fn from(payload: PayloadByte) -> Self {
        payload.value
    }
}
impl From<LenByte> for u8 {
    fn from(len: LenByte) -> Self {
        len.value
    }
}
impl From<LenByte> for usize {
    fn from(len: LenByte) -> Self {
        len.value as usize
    }
}
