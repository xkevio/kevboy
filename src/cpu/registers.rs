#[allow(non_snake_case)]
#[derive(Default)]
pub struct Registers {
    pub A: u8,
    pub F: u8,
    pub B: u8,
    pub C: u8,
    pub D: u8,
    pub E: u8,
    pub H: u8,
    pub L: u8,
    pub SP: u16,
    pub PC: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum Flag {
    Zero,
    Substraction,
    HalfCarry,
    Carry,
}

#[derive(Debug, Clone, Copy)]
pub enum Regs {
    AF,
    BC,
    DE,
    HL,
    SP,
}

impl Registers {
    /// Sets up register values after booting, so running the BOOT ROM is not needed.
    ///
    /// `header_checksum` is `$014D` and is responsible for the Carry and Half-Carry flag.
    pub fn new_dmg(header_checksum: u8) -> Self {
        let flag = if header_checksum == 0x00 {
            0b10000000
        } else {
            0b10110000
        };

        Self {
            A: 0x01,
            F: flag,
            B: 0x00,
            C: 0x13,
            D: 0x00,
            E: 0xD8,
            H: 0x01,
            L: 0x4D,
            SP: 0xFFFE,
            PC: 0x100,
        }
    }

    pub fn load_header_checksum(&mut self, header_checksum: u8) {
        let flag = if header_checksum == 0x00 {
            0b10000000
        } else {
            0b10110000
        };

        self.F = flag;
    }

    pub fn get_af(&self) -> u16 {
        (self.A as u16) << 8 | self.F as u16
    }

    pub fn get_bc(&self) -> u16 {
        (self.B as u16) << 8 | self.C as u16
    }

    pub fn get_de(&self) -> u16 {
        (self.D as u16) << 8 | self.E as u16
    }

    pub fn get_hl(&self) -> u16 {
        (self.H as u16) << 8 | self.L as u16
    }

    pub fn set_af(&mut self, value: u16) {
        let bytes = value.to_be_bytes();

        self.A = bytes[0];
        self.F = bytes[1];
    }

    pub fn set_bc(&mut self, value: u16) {
        let bytes = value.to_be_bytes();

        self.B = bytes[0];
        self.C = bytes[1];
    }

    pub fn set_de(&mut self, value: u16) {
        let bytes = value.to_be_bytes();

        self.D = bytes[0];
        self.E = bytes[1];
    }

    pub fn set_hl(&mut self, value: u16) {
        let bytes = value.to_be_bytes();

        self.H = bytes[0];
        self.L = bytes[1];
    }

    // sets the lower bits (4-7) of the AF register (F) according to a flag
    pub fn set_flag(&mut self, flag: Flag, bit: bool) {
        if bit {
            self.set_flag_bit(flag);
        } else {
            self.unset_flag_bit(flag);
        }
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Zero => (self.F & (1 << 7)) != 0,
            Flag::Substraction => (self.F & (1 << 6)) != 0,
            Flag::HalfCarry => (self.F & (1 << 5)) != 0,
            Flag::Carry => (self.F & (1 << 4)) != 0,
        }
    }

    fn set_flag_bit(&mut self, flag: Flag) {
        match flag {
            Flag::Zero => self.F |= 1 << 7,
            Flag::Substraction => self.F |= 1 << 6,
            Flag::HalfCarry => self.F |= 1 << 5,
            Flag::Carry => self.F |= 1 << 4,
        }
    }

    fn unset_flag_bit(&mut self, flag: Flag) {
        match flag {
            Flag::Zero => self.F &= !(1 << 7),
            Flag::Substraction => self.F &= !(1 << 6),
            Flag::HalfCarry => self.F &= !(1 << 5),
            Flag::Carry => self.F &= !(1 << 4),
        }
    }
}
