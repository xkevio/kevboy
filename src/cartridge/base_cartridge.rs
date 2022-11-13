use crate::cartridge::mbc::mbc1::MBC1;
use crate::mmu::mmio::MMIO;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, PartialEq)]
pub enum CartridgeType {
    NoMBC,
    MBC1(MBC1),
    MBC2,
    MBC3,
    MBC5,
    MBC7,
}

pub struct Cartridge {
    pub cartridge_type: CartridgeType,

    pub rom_bank_0: [u8; 0x4000],
    pub rom_bank_x: [u8; 0x4000],
    pub external_ram: [u8; 0x2000],

    rom_size: usize,
    ram_size: u8,
}

impl Cartridge {
    pub fn new(cartridge_type: CartridgeType, rom_size: usize, ram_size: u8) -> Self {
        Self {
            cartridge_type,
            rom_size,
            ram_size,
            ..Default::default()
        }
    }
}

impl Default for Cartridge {
    fn default() -> Self {
        Self {
            cartridge_type: CartridgeType::NoMBC,

            rom_bank_0: [0xFF; 0x4000],
            rom_bank_x: [0xFF; 0x4000],
            external_ram: [0xFF; 0x2000],

            rom_size: 0,
            ram_size: 0,
        }
    }
}

impl MMIO for Cartridge {
    fn read(&mut self, address: u16) -> u8 {
        match self.cartridge_type {
            CartridgeType::NoMBC => {
                if address < 0x4000 {
                    self.rom_bank_0[address as usize]
                } else {
                    self.rom_bank_x[address as usize - 0x4000]
                }
            }
            CartridgeType::MBC1(mut mbc1) => mbc1.read(address),
            _ => 0xFF
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match self.cartridge_type {
            CartridgeType::MBC1(mut mbc1) => mbc1.write(address, value),
            _ => {}
        }
    }
}
