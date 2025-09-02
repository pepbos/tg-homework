use crate::{Crc, MAX_FRAME_LEN, START, find_magic_byte};

/// Encoded message:
/// - START
/// - MagicByte
/// - Length
/// - Data
/// - Crc
pub fn encode_in_place(data: &[u8], out: &mut [u8]) -> usize {
    // Some size checks first.
    assert!(
        data.len() <= MAX_FRAME_LEN as usize,
        "data size = {} must not exceed max allowed size = {}",
        data.len(),
        MAX_FRAME_LEN
    );
    let encoded_len = encoded_len(data);
    assert!(
        out.len() >= encoded_len,
        "encoded data size = {} is larger than output buffer size = {}",
        encoded_len,
        out.len()
    );

    // Compute the magic byte that replaces all occurances of the START byte.
    let magic_byte = find_magic_byte(data);

    // Initialize the crc computation.
    let mut crc = Crc::new();

    // Start writing bytes.
    let mut iter = out.iter_mut();
    *iter.next().unwrap() = START;

    {
        // Helper for encoding the magic byte, updating the crc, and writing to the output buffer.
        let mut set_next = |byte: u8| {
            crc.process_encoded_byte(byte);
            *iter.next().unwrap() = byte;
        };

        set_next(magic_byte.into());
        set_next(data.len() as u8);
        for &byte in data.iter() {
            set_next(if byte == START { magic_byte } else { byte });
        }
    }

    // Finally write the crc.
    {
        let byte = crc.compute();
        *iter.next().unwrap() = if byte == START { magic_byte } else { byte };
    }

    // Return number of bytes written.
    encoded_len
}

pub fn encoded_len(data: &[u8]) -> usize {
    data.len() + 4
}
