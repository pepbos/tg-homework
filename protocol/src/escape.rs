use crate::{Error, SYNC, frame::HEADER_LEN};
use bitvec::prelude::*;

// Number of byte kinds in a frame (excluding the SYNC and ESCAPE bytes).
const MAX_BYTE_KINDS: u8 = crate::MAX_FRAME_LEN + HEADER_LEN as u8 - 2;

type Storage = u8; // TODO use u32

pub struct EscapeState {
    possible_bytes: BitArr!(for MAX_BYTE_KINDS as usize, in Storage, Lsb0),
    count: usize,
}

impl EscapeState {
    pub fn new() -> Self {
        if MAX_BYTE_KINDS >= SYNC {
            panic!(
                "Bug: cannot guarantee existence of escape character for max payload size ({}) and header size ({})",
                crate::MAX_FRAME_LEN,
                HEADER_LEN
            );
        }
        Self {
            possible_bytes: bitarr![Storage, Lsb0; 1; MAX_BYTE_KINDS as usize],
            count: 0,
        }
    }

    pub fn digest(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            if byte >= MAX_BYTE_KINDS {
                continue;
            }
            self.possible_bytes.set(byte as usize, false);
            self.count += 1;
            assert!(self.count <= MAX_BYTE_KINDS as usize, "Attempted to digest more than {} bytes", MAX_BYTE_KINDS);
        }
    }

    pub fn finalize(self) -> Escape {
        for (byte, is_possible) in self.possible_bytes.iter().by_vals().enumerate() {
            if is_possible {
                return Escape::try_from_raw(byte as u8).unwrap();
            }
        }
        Escape::try_from_raw(MAX_BYTE_KINDS).unwrap()
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct Escape {
    value: u8,
}

fn ensure_not_sync(byte: u8) -> Result<(), Error> {
    if byte == SYNC {
        Err(Error::EarlySync)
    } else {
        Ok(())
    }
}

impl Escape {
    pub fn try_from_raw(value: u8) -> Result<Self, Error> {
        ensure_not_sync(value)?;
        Ok(Self { value })
    }

    pub fn escape_byte(&self, byte: u8) -> u8 {
        if byte == self.value {
            panic!("Escape character ({}) encountered during escape", byte);
        }
        if byte == SYNC { self.value } else { byte }
    }

    pub fn escape_in_place(&self, bytes: &mut [u8]) {
        for byte in bytes.iter_mut() {
            *byte = self.escape_byte(*byte);
        }
    }

    pub fn unescape(&self, encoded: u8) -> Result<u8, Error> {
        ensure_not_sync(encoded)?;
        Ok(if encoded == self.value { SYNC } else { encoded })
    }
}

impl From<Escape> for u8 {
    fn from(value: Escape) -> Self {
        value.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_escape() {
        let arr = bitarr![u32, Lsb0; 0; 80];

        use rand::prelude::*;
        let mut rng = rand::rng();

        // Create some data bytes.
        let mut bytes = [0u8; MAX_BYTE_KINDS as usize];
        for byte in bytes.iter_mut() {
            // TODO do not use random numbers in test.
            *byte = rng.random::<u8>();
        }

        // Find the escape character.
        let escape: u8 = {
            let mut escape = EscapeState::new();
            escape.digest(&bytes);
            escape.finalize().into()
        };

        // Test that the magic byte is indeed not part of our data.
        for byte in bytes {
            assert_ne!(escape, byte);
        }
    }
}
