#[derive(Copy, Clone, Debug)]
pub struct Crc {
    crc16: crc16::State<crc16::ARC>,
}

impl Crc {
    pub fn new() -> Self {
        Self {crc16: crc16::State::new()}
    }

    pub fn update(&mut self, byte: u8) -> u8 {
        self.crc16.update(&[byte]);
        byte
    }

    pub fn digest(&mut self, bytes: &[u8]) {
        self.crc16.update(bytes);
    }

    pub fn finalize(self) -> [u8; 2] {
        self.crc16.get().to_be_bytes()
    }
}
