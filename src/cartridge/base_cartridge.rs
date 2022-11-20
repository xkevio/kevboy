use crate::cartridge::mbc::mbc1::MBC1;
use crate::cartridge::mbc::mbc5::MBC5;
use crate::cartridge::mbc::no_mbc::NoMBC;
use crate::mmu::mmio::MMIO;

#[allow(clippy::upper_case_acronyms)]
#[derive(PartialEq)]
pub enum CartridgeType {
    NoMBC(NoMBC),
    MBC1(MBC1),
    MBC2,
    MBC3,
    MBC5(MBC5),
    MBC7,
}

pub struct Cartridge {
    pub cartridge_type: CartridgeType,
    pub title: String,
}

impl Cartridge {
    pub fn new(cartridge_type: CartridgeType, title: &str) -> Self {
        Self {
            cartridge_type,
            title: title.to_string(),
        }
    }
}

impl Default for Cartridge {
    fn default() -> Self {
        Self {
            cartridge_type: CartridgeType::NoMBC(NoMBC::new(&[])),
            title: String::from(""),
        }
    }
}

impl MMIO for Cartridge {
    fn read(&mut self, address: u16) -> u8 {
        match &mut self.cartridge_type {
            CartridgeType::NoMBC(nombc) => nombc.read(address),
            CartridgeType::MBC1(mbc1) => mbc1.read(address),
            CartridgeType::MBC5(mbc5) => mbc5.read(address),
            _ => 0xFF,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match &mut self.cartridge_type {
            CartridgeType::MBC1(mbc1) => mbc1.write(address, value),
            CartridgeType::MBC5(mbc5) => mbc5.write(address, value),
            _ => {}
        }
    }
}
