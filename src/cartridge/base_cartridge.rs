use crate::mmu::mmio::MMIO;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum CartridgeType {
    NoMBC,
    MBC1,
    MBC2,
    MBC3,
    MBC5,
    MBC7,
}

pub struct Cartridge {
    cartridge_type: CartridgeType,
    rom_size: u32,
    ram_size: u16,
}

impl MMIO for Cartridge {
    fn read(&mut self, address: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, address: u16, value: u8) {
        todo!()
    }
}
