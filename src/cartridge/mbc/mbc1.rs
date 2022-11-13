use crate::mmu::mmio::MMIO;

#[derive(Clone, Copy, PartialEq)]
pub struct MBC1 {
    pub ram_enable: bool,
    pub rom_bank_number: u8,
    pub ram_or_upper_rom: u8,
    pub banking_mode: u8,
}

impl Default for MBC1 {
    fn default() -> Self {
        Self {
            ram_enable: false,
            rom_bank_number: 0x1,
            ram_or_upper_rom: 0,
            banking_mode: 0,
        }
    }
}

impl MMIO for MBC1 {
    // cartridge registers are write only
    fn read(&mut self, _address: u16) -> u8 {
        unimplemented!()
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                if value & 0xF == 0xA {
                    self.ram_enable = true;
                } else {
                    self.ram_enable = false;
                }
            }
            0x2000..=0x3FFF => {
                let rom_bank = if value & 0x1F == 0 { 1 } else { value & 0x1F };

                // check if bank number higher than number in cart and mask
            }
            0x4000..=0x5FFF => {
                // TODO: size check
                self.ram_or_upper_rom = value & 0x3;
            }
            0x6000..=0x7FFF => {
                self.banking_mode = value & 0x1;
            }
            _ => unreachable!("Faulty match ranges"),
        }
    }
}
