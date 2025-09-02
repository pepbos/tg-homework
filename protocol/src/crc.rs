#[derive(Copy, Clone)]
pub struct Crc {}

impl Default for Crc {
    fn default() -> Self {
        todo!()
    }
}

impl Crc {
    pub fn update(&mut self, byte: u8) -> u8 {
        todo!();
    }

    pub fn finalize(self) -> u8 {
        todo!();
    }
}
