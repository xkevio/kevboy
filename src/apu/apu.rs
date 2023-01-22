use crate::mmu::mmio::MMIO;

#[allow(clippy::upper_case_acronyms)]
pub struct APU {
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,

    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,

    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,

    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,

    nr50: u8,
    nr51: u8,
    nr52: u8,
}

impl Default for APU {
    fn default() -> Self {
        Self {
            nr10: 0x80,
            nr11: 0xBF,
            nr12: 0xF3,
            nr13: 0xFF,
            nr14: 0xBF,

            nr21: 0x3F,
            nr22: 0x00,
            nr23: 0xFF,
            nr24: 0xBF,

            nr30: 0x7F,
            nr31: 0xFF,
            nr32: 0x9F,
            nr33: 0xFF,
            nr34: 0xBF,

            nr41: 0xFF,
            nr42: 0x00,
            nr43: 0x00,
            nr44: 0xBF,

            nr50: 0x77,
            nr51: 0xF3,
            nr52: 0xF1,
        }
    }
}

impl MMIO for APU {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0xFF10 => self.nr10,
            0xFF11 => self.nr11,
            0xFF12 => self.nr12,
            0xFF13 => self.nr13,
            0xFF14 => self.nr14,
            0xFF16 => self.nr21,
            0xFF17 => self.nr22,
            0xFF18 => self.nr23,
            0xFF19 => self.nr24,
            0xFF1A => self.nr30,
            0xFF1B => self.nr31,
            0xFF1C => self.nr32,
            0xFF1D => self.nr33,
            0xFF1E => self.nr34,
            0xFF20 => self.nr41,
            0xFF21 => self.nr42,
            0xFF22 => self.nr43,
            0xFF23 => self.nr44,
            0xFF24 => self.nr50,
            0xFF25 => self.nr51,
            0xFF26 => self.nr52,
            _ => 0xFF,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF10 => self.nr10 = value | 0b1000_0000,
            0xFF11 => self.nr11 = value,
            0xFF12 => self.nr12 = value,
            0xFF13 => self.nr13 = value,
            0xFF14 => self.nr14 = value | 0b0011_1000,
            0xFF16 => self.nr21 = value,
            0xFF17 => self.nr22 = value,
            0xFF18 => self.nr23 = value,
            0xFF19 => self.nr24 = value | 0b0011_1000,
            0xFF1A => self.nr30 = value | 0b0111_1111,
            0xFF1B => self.nr31 = value,
            0xFF1C => self.nr32 = value | 0b1001_1111,
            0xFF1D => self.nr33 = value,
            0xFF1E => self.nr34 = value | 0b0011_1000,
            0xFF20 => self.nr41 = value | 0b1100_0000,
            0xFF21 => self.nr42 = value,
            0xFF22 => self.nr43 = value,
            0xFF23 => self.nr44 = value | 0b0011_1111,
            0xFF24 => self.nr50 = value,
            0xFF25 => self.nr51 = value,
            0xFF26 => self.nr52 = value | 0b0111_0000,
            _ => {}
        }
    }
}
