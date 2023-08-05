use crate::mmu::mmio::MMIO;

#[derive(PartialEq, Eq)]
pub struct MBC1 {
    pub rom: Vec<u8>,
    pub external_ram: Vec<[u8; 0x2000]>,

    rom_size: usize,
    ram_size: u8,

    ram_enable: bool,
    rom_bank_number: u8,
    ram_or_upper_rom: u8,
    banking_mode: u8,
}

impl MBC1 {
    pub fn new(rom: &[u8], rom_size: usize, ram_size: u8) -> Self {
        Self {
            rom: rom.to_vec(),
            external_ram: vec![[0xFF; 0x2000]; (ram_size / 8) as usize],

            rom_size,
            ram_size,

            ram_enable: false,
            rom_bank_number: 0x01,
            ram_or_upper_rom: 0x00,
            banking_mode: 0x00,
        }
    }
}

impl MMIO for MBC1 {
    #[inline(always)]
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => {
                if self.banking_mode == 0 || self.rom_size < 1024 {
                    self.rom[address as usize]
                } else {
                    let rom_bank = self.ram_or_upper_rom << 5;
                    let address = (rom_bank as u32 * 0x4000) + address as u32;

                    self.rom[address as usize & (self.rom.len() - 1)]
                }
            }
            0x4000..=0x7FFF => {
                let rom_bank = if self.rom_size < 1024 {
                    self.rom_bank_number
                } else {
                    (self.ram_or_upper_rom << 5) | self.rom_bank_number
                };

                let address = (rom_bank as u32 * 0x4000) + (address as u32 - 0x4000);
                self.rom[address as usize & (self.rom.len() - 1)]
            }
            0xA000..=0xBFFF => {
                if self.ram_enable && self.ram_size > 0 {
                    if self.banking_mode == 0 || self.ram_size <= 8 {
                        self.external_ram[0][address as usize - 0xA000]
                    } else {
                        self.external_ram[self.ram_or_upper_rom as usize][address as usize - 0xA000]
                    }
                } else {
                    0xFF
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline(always)]
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
                let max_banks_bits = self.rom_size / 16;

                // check if bank number higher than number in cart and mask
                if (rom_bank as usize) >= max_banks_bits {
                    self.rom_bank_number = rom_bank & ((max_banks_bits - 1) as u8);
                } else {
                    self.rom_bank_number = rom_bank;
                }
            }
            0x4000..=0x5FFF => {
                if self.rom_size >= 1024 || self.ram_size >= 32 {
                    self.ram_or_upper_rom = value & 0x3;
                }
            }
            0x6000..=0x7FFF => {
                self.banking_mode = value & 0x1;
            }
            0xA000..=0xBFFF => {
                if self.ram_enable && self.ram_size > 0 {
                    if self.banking_mode == 0 {
                        self.external_ram[0][address as usize - 0xA000] = value;
                    } else {
                        self.external_ram[self.ram_or_upper_rom as usize]
                            [address as usize - 0xA000] = value;
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
