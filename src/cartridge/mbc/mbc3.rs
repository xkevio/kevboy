use crate::mmu::mmio::MMIO;

#[derive(PartialEq, Eq)]
pub struct MBC3 {
    pub rom: Vec<u8>,
    pub external_ram: Vec<[u8; 0x2000]>,

    rtc: RealTimeClock,

    ram_timer_enable: bool,
    rom_bank_number: u8,
    ram_bank_rtc: u8,
    latch_data: u8,
}

#[derive(Default, PartialEq, Eq)]
struct RealTimeClock {
    seconds: u8,
    minutes: u8,
    hours: u8,

    dl: u8,
    dh: u8,
}

impl MMIO for RealTimeClock {
    fn read(&mut self, address: u16) -> u8 {
        // no masking on read as we mask on write already
        match address {
            0x08 => self.seconds,
            0x09 => self.minutes,
            0x0A => self.hours,
            0x0B => self.dl,
            0x0C => self.dh,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x08 => self.seconds = value & 0x3B,
            0x09 => self.minutes = value & 0x3B,
            0x0A => self.hours = value & 0x17,
            0x0B => self.dl = value,
            0x0C => self.dh = value,
            _ => unreachable!(),
        }
    }
}

impl MBC3 {
    pub fn new(rom: &[u8]) -> Self {
        MBC3 {
            rom: rom.to_vec(),
            external_ram: vec![[0xFF; 0x2000]; 4],

            rtc: RealTimeClock::default(),

            ram_timer_enable: false,
            rom_bank_number: 0x01,
            ram_bank_rtc: 0x00,
            latch_data: 0x00,
        }
    }
}

impl MMIO for MBC3 {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => {
                let address = (self.rom_bank_number as u32) * 0x4000 + (address as u32 - 0x4000);
                self.rom[address as usize]
            }
            0xA000..=0xBFFF => {
                if self.ram_timer_enable {
                    if self.ram_bank_rtc <= 0x03 {
                        self.external_ram[(self.ram_bank_rtc & 0x03) as usize]
                            [(address - 0xA000) as usize]
                    } else if self.ram_bank_rtc >= 0x08 && self.ram_bank_rtc <= 0x0C {
                        self.rtc.read(self.ram_bank_rtc as u16)
                    } else {
                        0xFF
                    }
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
                if value == 0x0A {
                    self.ram_timer_enable = true;
                } else {
                    self.ram_timer_enable = false;
                }
            }
            0x2000..=0x3FFF => {
                self.rom_bank_number = value & 0x7F;
            }
            0x4000..=0x5FFF => {
                // 0x00 - 0x03 -> RAM, 0x08 - 0x0C -> RTC
                self.ram_bank_rtc = value;
            }
            0x6000..=0x7FFF => {
                // TODO: Latch Clock Data
            }
            0xA000..=0xBFFF => {
                if self.ram_timer_enable {
                    if self.ram_bank_rtc <= 0x03 {
                        self.external_ram[(self.ram_bank_rtc & 0x03) as usize]
                            [(address - 0xA000) as usize] = value;
                    } else {
                        if self.ram_bank_rtc >= 0x08 && self.ram_bank_rtc <= 0x0C {
                            self.rtc.write(self.ram_bank_rtc as u16, value);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
