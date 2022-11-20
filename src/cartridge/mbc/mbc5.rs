use crate::mmu::mmio::MMIO;

#[derive(PartialEq)]
pub struct MBC5 {
    pub rom: Vec<u8>,
    pub external_ram: Vec<[u8; 0x2000]>,

    rom_size: usize,
    ram_size: u8,

    ram_enable: bool,
    rom_bank_number: u8,
    rom_bank_bit9: u8,
    ram_bank_number: u8,
}

impl MBC5 {
    pub fn new(rom: &[u8], rom_size: usize, ram_size: u8) -> Self {
        Self {
            rom: rom.to_vec(),
            external_ram: vec![[0xFF; 0x2000]; (ram_size / 8) as usize],

            rom_size,
            ram_size,

            ram_enable: false,
            rom_bank_number: 0x01,
            rom_bank_bit9: 0x00,
            ram_bank_number: 0x00,
        }
    }
}

impl MMIO for MBC5 {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => {
                let rom_bank: u16 =
                    ((self.rom_bank_bit9 as u16) << 8) | (self.rom_bank_number as u16);

                let address = (rom_bank as u32 * 0x4000) + (address as u32 - 0x4000);
                self.rom[address as usize & self.rom.len() - 1]
            }
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    self.external_ram[self.ram_bank_number as usize][address as usize - 0xA000]
                } else {
                    0xFF
                }
            }
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                // 0x0A for enable, 0x00 for disable (technically any other value)
                if value == 0x0A {
                    self.ram_enable = true;
                } else {
                    self.ram_enable = false;
                }
            }
            0x2000..=0x2FFF => {
                self.rom_bank_number = value;
            }
            0x3000..=0x3FFF => {
                self.rom_bank_bit9 = value & 0x1;
            }
            0x4000..=0x5FFF => {
                self.ram_bank_number = value & 0x0F;
            }
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    self.external_ram[self.ram_bank_number as usize][address as usize - 0xA000] =
                        value;
                }
            }
            _ => {}
        }
    }
}
