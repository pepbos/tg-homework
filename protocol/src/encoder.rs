use crate::{frame::HEADER_LEN, CrcState, Escape, Header, MAX_FRAME_LEN, SYNC};

pub fn encoded_len(data: &[u8]) -> usize {
    data.len() + HEADER_LEN
}

pub fn encode_in_place(id: u8, data: &[u8], out: &mut [u8]) -> Header {
    // Output buffer size check.
    let encoded_len = encoded_len(data);
    assert!(
        out.len() >= encoded_len,
        "encoded data size = {} is larger than output buffer size = {}",
        encoded_len,
        out.len()
    );

    let header = Header::new(id, data);
    let mut iter = out.iter_mut();

    *iter.next().unwrap() = header.sync;
    *iter.next().unwrap() = header.escape.into();

    let mut escape_and_push = |byte: u8| {
        *iter.next().unwrap() = header.escape.escape_byte(byte);
    };

    escape_and_push(header.id);
    escape_and_push(header.len);
    for &payload in data.iter() {
        escape_and_push(payload);
    }
    for &crc in header.crc.iter() {
        escape_and_push(crc);
    }

    header
}
