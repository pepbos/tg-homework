use crate::{Crc, MAX_FRAME_LEN, SYNC, find_escape};

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

    // Compute the escape character that replaces all occurances of the SYNC byte.
    let escape = find_escape(data);

    // Initialize the crc computation.
    let mut crc = Crc::default();

    // Start writing bytes.
    let mut iter = out.iter_mut();

    // Byte0. SYNC
    *iter.next().unwrap() = SYNC;

    {
        // Helper for encoding the magic byte, updating the crc, and writing to the output buffer.
        let mut set_next = |byte: u8| {
            crc.update(byte);
            *iter.next().unwrap() = byte;
        };

        // Byte1. ESCAPE
        set_next(escape.into());
        // Byte2. LEN
        set_next(data.len() as u8);
        // Byte3..Byte[3+LEN]. PAYLOAD
        for &byte in data.iter() {
            set_next(if byte == SYNC { escape } else { byte });
        }
    }

    // Byte[3+LEN, 4+LEN]. CRC
    for byte in crc.finalize() {
        *iter.next().unwrap() = if byte == SYNC { escape } else { byte };
    }

    // Return number of bytes written.
    encoded_len
}

pub fn encoded_len(data: &[u8]) -> usize {
    data.len() + 5
}
