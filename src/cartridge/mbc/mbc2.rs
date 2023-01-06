use crate::mmu::mmio::MMIO;

#[derive(PartialEq, Eq)]
pub struct MBC2 {
    pub rom: Vec<u8>,
    pub built_in_ram: [u8; 512],

    rom_bank: u8,
    ram_enable: bool,
}

impl MBC2 {
    pub fn new(rom: &[u8]) -> Self {
        Self {
            rom: rom.to_vec(),
            built_in_ram: [0xFF; 512],

            rom_bank: 0x01,
            ram_enable: false,
        }
    }
}

impl MMIO for MBC2 {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => {
                let address = (self.rom_bank as u32 * 0x4000) + (address as u32 - 0x4000);
                self.rom[address as usize & (self.rom.len() - 1)]
            }
            // upper 4 bits ignored but mooneye tests expects them to be open bus (1)
            0xA000..=0xA1FF => {
                if self.ram_enable {
                    0xF0 | (self.built_in_ram[(address - 0xA000) as usize] & 0xF)
                } else {
                    0xFF
                }
            }
            0xA200..=0xBFFF => {
                if self.ram_enable {
                    0xF0 | (self.built_in_ram[(address % 0x200) as usize] & 0xF)
                } else {
                    0xFF
                }
            }
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x3FFF => {
                let bit8 = (address & 0x100) >> 8;

                if bit8 == 0 {
                    if (value & 0xF) == 0xA {
                        self.ram_enable = true;
                    } else {
                        self.ram_enable = false;
                    }
                } else {
                    let rom_bank = value & 0xF;
                    self.rom_bank = if rom_bank == 0 { 1 } else { rom_bank };
                }
            }
            0xA000..=0xA1FF => {
                if self.ram_enable {
                    self.built_in_ram[(address - 0xA000) as usize] = value & 0xF;
                }
            }
            0xA200..=0xBFFF => {
                if self.ram_enable {
                    self.built_in_ram[(address % 0x200) as usize] = value & 0xF;
                }
            }
            _ => {}
        }
    }
}
