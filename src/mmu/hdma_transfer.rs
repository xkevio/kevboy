use super::mmio::MMIO;

pub struct Hdma {
    /// HDMA source (high, low)
    hdma1: u8,
    hdma2: u8,

    /// HDMA dest (high, low)
    hdma3: u8,
    hdma4: u8,

    /// VRAM DMA length/mode/start
    hdma5: u8,

    pub halted: bool,
    pub hdma_in_progress: bool,
    pub bytes: [u8; 0x10],

    pub transfer: bool,
}

impl Hdma {
    pub fn is_gdma(&self) -> bool {
        (self.hdma5 & 0x80) >> 7 == 0
    }

    #[inline(always)]
    pub fn source(&self) -> u16 {
        u16::from_be_bytes([self.hdma1, self.hdma2])
    }

    pub fn dest(&self) -> u16 {
        u16::from_be_bytes([self.hdma3, self.hdma4])
    }

    pub fn length(&self) -> u16 {
        ((self.hdma5 & 0x7F) as u16 + 1) * 0x10
    }

    pub fn update_remaining(&mut self) {
        let new_source = self.source() + 0x10;
        let new_dest = self.dest() + 0x10;

        self.hdma1 = new_source.to_be_bytes()[0];
        self.hdma2 = new_source.to_be_bytes()[1];

        self.hdma3 = new_dest.to_be_bytes()[0];
        self.hdma4 = new_dest.to_be_bytes()[1];

        self.hdma5 -= 1;
    }

    pub fn complete_transfer(&mut self) {
        self.hdma_in_progress = false;
        self.halted = false;
        self.hdma5 = 0xFF;
    }

    pub fn terminate_transfer(&mut self) {
        self.hdma_in_progress = false;
        self.halted = false;
        self.hdma5 = 0x80 | (self.hdma5 - 1);
    }
}

impl Default for Hdma {
    fn default() -> Self {
        Self {
            hdma1: 0xFF,
            hdma2: 0xFF,
            hdma3: 0xFF,
            hdma4: 0xFF,
            hdma5: 0xFF,
            halted: false,
            hdma_in_progress: false,
            bytes: [0xFF; 0x10],
            transfer: false,
        }
    }
}

impl MMIO for Hdma {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0xFF51 => self.hdma1,
            0xFF52 => self.hdma2,
            0xFF53 => self.hdma3,
            0xFF54 => self.hdma4,
            0xFF55 => self.hdma5,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF51 => self.hdma1 = value,
            0xFF52 => self.hdma2 = value & 0xF0,
            0xFF53 => self.hdma3 = value & 0x1F,
            0xFF54 => self.hdma4 = value & 0xF0,
            0xFF55 => self.hdma5 = value,
            _ => {}
        }
    }
}
