#[derive(Copy, Clone, Debug)]
pub struct Crc {
    crc16: crc16::State<crc16::ARC>,
}

impl Default for Crc {
    fn default() -> Self {
        Self {crc16: crc16::State::new()}
    }
}

impl Crc {
    pub fn update(&mut self, byte: u8) -> u8 {
        self.crc16.update(&[byte]);
        byte
    }

    pub fn finalize(self) -> [u8; 2] {
        self.crc16.get().to_be_bytes()
    }
}
