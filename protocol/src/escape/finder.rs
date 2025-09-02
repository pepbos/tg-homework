use bitvec::prelude::*;

use crate::SYNC;

const MAX_FRAME_LEN: usize = crate::MAX_FRAME_LEN as usize;

type Storage = u8;

struct EscapeFinder {
    possible_bytes: BitArr!(for MAX_FRAME_LEN, in Storage, Lsb0),
}

impl Default for EscapeFinder {
    fn default() -> Self {
        Self {
            possible_bytes: bitarr![Storage, Lsb0; 1; MAX_FRAME_LEN],
        }
    }
}

impl EscapeFinder {
    #[inline]
    fn exclude_byte(&mut self, byte: u8) {
        if byte >= crate::MAX_FRAME_LEN {
            return;
        }
        self.possible_bytes.set(byte as usize, false);
    }

    #[inline]
    fn find_escape(self) -> u8 {
        for (byte, is_possible) in self.possible_bytes.iter().by_vals().enumerate() {
            if is_possible {
                return byte as u8;
            }
        }
        crate::MAX_FRAME_LEN
    }
}

pub fn find_escape(data: &[u8]) -> u8 {
    assert!(
        data.len() <= MAX_FRAME_LEN,
        "data length must not exceed {}, got len = {}",
        MAX_FRAME_LEN,
        data.len()
    );
    let mut finder = EscapeFinder::default();
    for &byte in data {
        finder.exclude_byte(byte);
    }
    finder.find_escape()
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
        let mut bytes = [0u8; super::MAX_FRAME_LEN];
        for byte in bytes.iter_mut() {
            // TODO do not use random numbers in test.
            *byte = rng.random::<u8>();
        }

        // Find the escape character.
        let escape = find_escape(&bytes);
        // Test that the magic byte is indeed not part of our data.
        for byte in bytes {
            assert_ne!(escape, byte);
        }
    }
}
