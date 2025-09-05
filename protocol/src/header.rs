use crate::{CrcState, Escape, EscapeState, MAX_FRAME_LEN, SYNC};

pub const HEADER_LEN: usize = 6;

#[derive(Clone, Debug)]
pub struct Header {
    pub(crate) sync: u8,
    pub(crate) escape: Escape,
    pub(crate) id: u8,
    pub(crate) len: u8,
    pub(crate) crc: [u8; 2],
}

impl Header {
    pub fn get_len(&self) -> usize {
        self.len as usize + HEADER_LEN
    }

    pub fn new(id: u8, data: &[u8]) -> Self {
        let sync = SYNC;
        let len = {
            // length checks
            assert!(
                data.len() <= MAX_FRAME_LEN as usize,
                "data size = {} must not exceed max allowed size = {}",
                data.len(),
                MAX_FRAME_LEN
            );
            data.len() as u8
        };

        let crc: [u8; 2] = {
            let mut crc = CrcState::new();
            crc.digest_single(id);
            crc.digest_single(len);
            crc.digest(data);
            crc.finalize()
        };

        let escape = {
            let mut escape = EscapeState::new();
            escape.digest(&[id]);
            escape.digest(&[len]);
            escape.digest(data);
            escape.digest(&crc);
            escape.finalize()
        };

        Self {
            sync,
            escape,
            id,
            len,
            crc,
        }
    }
}
