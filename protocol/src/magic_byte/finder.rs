use bitvec::prelude::*;

use crate::START;

const MAX_FRAME_LEN: usize = crate::MAX_FRAME_LEN as usize;

type Storage = u8;

struct MagicByteFinder {
    possible_bytes: BitArr!(for MAX_FRAME_LEN, in Storage, Lsb0),
}

impl Default for MagicByteFinder {
    fn default() -> Self {
        Self {
            possible_bytes: bitarr![Storage, Lsb0; 1; MAX_FRAME_LEN],
        }
    }
}

impl MagicByteFinder {
    #[inline]
    fn exclude_byte(&mut self, byte: u8) {
        if byte >= crate::MAX_FRAME_LEN {
            return;
        }
        self.possible_bytes.set(byte as usize, false);
    }

    #[inline]
    fn find_magic_byte(self) -> u8 {
        for (byte, is_possible) in self.possible_bytes.iter().by_vals().enumerate() {
            if is_possible {
                return byte as u8;
            }
        }
        crate::MAX_FRAME_LEN
    }
}

pub fn find_magic_byte(data: &[u8]) -> u8 {
    assert!(
        data.len() <= MAX_FRAME_LEN,
        "data length must not exceed {}, got len = {}",
        MAX_FRAME_LEN,
        data.len()
    );
    let mut finder = MagicByteFinder::default();
    for &byte in data {
        finder.exclude_byte(byte);
    }
    finder.find_magic_byte()
}

#[cfg(test)]
mod tests {
    use super::find_magic_byte;
    use bitvec::prelude::*;

    #[test]
    fn test_find_magic_byte() {
        let arr = bitarr![u32, Lsb0; 0; 80];

        use rand::prelude::*;
        let mut rng = rand::rng();

        // Create some data bytes.
        let mut bytes = [0u8; super::MAX_FRAME_LEN];
        for byte in bytes.iter_mut() {
            // TODO do not use random numbers in test.
            *byte = rng.random::<u8>();
        }

        // Find the magic byte.
        let magic_byte = find_magic_byte(&bytes);
        // Test that the magic byte is indeed not part of our data.
        for byte in bytes {
            assert_ne!(magic_byte, byte);
        }
    }
}
