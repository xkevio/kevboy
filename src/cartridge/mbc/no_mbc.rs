use crate::mmu::mmio::MMIO;

#[derive(PartialEq)]
pub struct NoMBC {
    pub rom: Vec<u8>,
}

impl NoMBC {
    pub fn new(rom: &[u8]) -> Self {
        Self { rom: rom.to_vec() }
    }
}

impl MMIO for NoMBC {
    fn read(&mut self, address: u16) -> u8 {
        if address < 0x8000 {
            self.rom[address as usize]
        } else {
            0xFF
        }
    }

    fn write(&mut self, _address: u16, _value: u8) {
        unimplemented!("MBC0 doesn't write to ROM or RAM")
    }
}
