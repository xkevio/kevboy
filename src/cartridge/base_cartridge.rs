use crate::cartridge::mbc::mbc1::MBC1;
use crate::cartridge::mbc::mbc2::MBC2;
use crate::cartridge::mbc::mbc3::MBC3;
use crate::cartridge::mbc::mbc5::MBC5;
use crate::cartridge::mbc::no_mbc::NoMBC;
use crate::mmu::mmio::MMIO;

#[allow(clippy::upper_case_acronyms)]
#[derive(PartialEq, Eq)]
pub enum CartridgeType {
    NoMBC(NoMBC),
    MBC1(MBC1),
    MBC2(MBC2),
    MBC3(MBC3),
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

    // TODO: create per_cartridge! macro to avoid repetition
    /// Loads in a `.sav` file and puts its contents
    /// into cartridge RAM
    pub fn load_sram(&mut self, save: &[u8]) {
        match &mut self.cartridge_type {
            CartridgeType::MBC1(mbc1) => {
                for (i, bank) in save.chunks(0x2000).enumerate() {
                    mbc1.external_ram[i] = bank.try_into().unwrap();
                }
            }
            CartridgeType::MBC2(mbc2) => {
                mbc2.built_in_ram = save.try_into().unwrap();
            }
            CartridgeType::MBC3(mbc3) => {
                for (i, bank) in save.chunks(0x2000).enumerate() {
                    mbc3.external_ram[i] = bank.try_into().unwrap();
                }
            }
            CartridgeType::MBC5(mbc5) => {
                for (i, bank) in save.chunks(0x2000).enumerate() {
                    mbc5.external_ram[i] = bank.try_into().unwrap();
                }
            }
            _ => {}
        }
    }

    /// Dumps all of SRAM into a Vec of bytes by joining
    /// the banks together.
    ///
    /// Returns `None` if no cartridge RAM is present.
    pub fn dump_sram(&self) -> Option<Vec<u8>> {
        match &self.cartridge_type {
            CartridgeType::MBC1(mbc1) => Some(mbc1.external_ram.concat()),
            CartridgeType::MBC2(mbc2) => Some(mbc2.built_in_ram.to_vec()),
            CartridgeType::MBC3(mbc3) => Some(mbc3.external_ram.concat()),
            CartridgeType::MBC5(mbc5) => Some(mbc5.external_ram.concat()),
            _ => None,
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

// This could be optimized, currently: enum dispatch (still better than dynamic dispatch).
impl MMIO for Cartridge {
    fn read(&mut self, address: u16) -> u8 {
        match &mut self.cartridge_type {
            CartridgeType::NoMBC(nombc) => nombc.read(address),
            CartridgeType::MBC1(mbc1) => mbc1.read(address),
            CartridgeType::MBC2(mbc2) => mbc2.read(address),
            CartridgeType::MBC3(mbc3) => mbc3.read(address),
            CartridgeType::MBC5(mbc5) => mbc5.read(address),
            _ => 0xFF,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match &mut self.cartridge_type {
            CartridgeType::MBC1(mbc1) => mbc1.write(address, value),
            CartridgeType::MBC2(mbc2) => mbc2.write(address, value),
            CartridgeType::MBC3(mbc3) => mbc3.write(address, value),
            CartridgeType::MBC5(mbc5) => mbc5.write(address, value),
            _ => {}
        }
    }
}
