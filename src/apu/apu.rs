use crate::mmu::mmio::MMIO;

// WAVE DUTY CYCLES
const WAVE_12_5: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];
const WAVE_25: [u8; 8] = [0, 0, 0, 0, 0, 0, 1, 1];
const WAVE_50: [u8; 8] = [0, 0, 0, 0, 1, 1, 1, 1];
const WAVE_75: [u8; 8] = [1, 1, 1, 1, 1, 1, 0, 0];

/// Channel 1 produces square waves and uses both envelope and sweep
/// functionality. Uses the wave constants above to produce said signals.
struct ChannelOne {
    pub buffer: Vec<u8>,
    wave_duty_pos: u8,

    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
}

impl Default for ChannelOne {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            wave_duty_pos: 0,

            nr10: 0x80,
            nr11: 0xBF,
            nr12: 0xF3,
            nr13: 0xFF,
            nr14: 0xBF,
        }
    }
}

impl ChannelOne {
    pub fn clear(&mut self) {
        self.nr10 = 0;
        self.nr11 = 0;
        self.nr12 = 0;
        self.nr13 = 0;
        self.nr14 = 0;
    }
}

/// Channel 2 produces square waves and uses just the envelope functionality.
/// Uses the wave constants above to produce said signals.
struct ChannelTwo {
    pub buffer: Vec<u8>,
    wave_duty_pos: u8,

    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,
}

impl Default for ChannelTwo {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            wave_duty_pos: 0,

            nr21: 0x3F,
            nr22: 0x00,
            nr23: 0xFF,
            nr24: 0xBF,
        }
    }
}

impl ChannelTwo {
    pub fn tick(&mut self, div_apu: u8) {
        // TODO: tick length timer and update duty cycle
        // update envelope
        if div_apu % 2 == 0 {

        }
    }

    // TODO: do apu registers have unused bits set to 1?
    pub fn clear(&mut self) {
        self.nr21 = 0;
        self.nr22 = 0;
        self.nr23 = 0;
        self.nr24 = 0;
    }
}

/// Channel 3 can produce custom waves from 4 bit samples based on Wave RAM.
struct ChannelThree {
    pub buffer: Vec<u8>,

    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,
}

impl Default for ChannelThree {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            nr30: 0x7F,
            nr31: 0xFF,
            nr32: 0x9F,
            nr33: 0xFF,
            nr34: 0xBF,
        }
    }
}

impl ChannelThree {
    pub fn clear(&mut self) {
        self.nr30 = 0;
        self.nr31 = 0;
        self.nr32 = 0;
        self.nr33 = 0;
        self.nr34 = 0;
    }
}

/// Channel 4 can produce pseudo random noise and also uses envelope.
struct ChannelFour {
    pub buffer: Vec<u8>,

    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,
}

impl Default for ChannelFour {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            nr41: 0xFF,
            nr42: 0x00,
            nr43: 0x00,
            nr44: 0xBF,
        }
    }
}

impl ChannelFour {
    pub fn clear(&mut self) {
        self.nr41 = 0;
        self.nr42 = 0;
        self.nr43 = 0;
        self.nr44 = 0;
    }
}

/// The APU consists of four channels:
/// 
/// - **Channel 1:** Square waves (envelope + sweep)
/// - **Channel 2:** Square waves (envelope)
/// - **Channel 3:** Custom waves based on 4 bit samples in Wave RAM
/// - **Channel 4:** Noise (envelope)
/// 
/// Uses an internal `div_apu` timer based on bit 4 of DIV.
#[allow(clippy::upper_case_acronyms)]
pub struct APU {
    div_apu: u8,
    div_bit: u8,

    ch1: ChannelOne,
    ch2: ChannelTwo,
    ch3: ChannelThree,
    ch4: ChannelFour,

    // Global settings (master volume, panning, on/off)
    nr50: u8,
    nr51: u8,
    nr52: u8,
}

impl Default for APU {
    fn default() -> Self {
        Self {
            div_apu: 0,
            div_bit: 0,

            ch1: ChannelOne::default(),
            ch2: ChannelTwo::default(),
            ch3: ChannelThree::default(),
            ch4: ChannelFour::default(),

            nr50: 0x77,
            nr51: 0xF3,
            nr52: 0xF1,
        }
    }
}

impl MMIO for APU {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0xFF10 => self.ch1.nr10,
            0xFF11 => self.ch1.nr11,
            0xFF12 => self.ch1.nr12,
            0xFF13 => self.ch1.nr13,
            0xFF14 => self.ch1.nr14,

            0xFF16 => self.ch2.nr21,
            0xFF17 => self.ch2.nr22,
            0xFF18 => self.ch2.nr23,
            0xFF19 => self.ch2.nr24,

            0xFF1A => self.ch3.nr30,
            0xFF1B => self.ch3.nr31,
            0xFF1C => self.ch3.nr32,
            0xFF1D => self.ch3.nr33,
            0xFF1E => self.ch3.nr34,

            0xFF20 => self.ch4.nr41,
            0xFF21 => self.ch4.nr42,
            0xFF22 => self.ch4.nr43,
            0xFF23 => self.ch4.nr44,

            0xFF24 => self.nr50,
            0xFF25 => self.nr51,
            0xFF26 => self.nr52,
            _ => 0xFF,
        }
    }

    // TODO: mask on read not write?
    fn write(&mut self, address: u16, value: u8) {
        // NR52 is writable even with APU turned off
        if address == 0xFF26 {
            self.nr52 |= value & (1 << 7);

            if !self.is_apu_enabled() {
                self.ch1.clear();
                self.ch2.clear();
                self.ch3.clear();
                self.ch4.clear();
            }
        }

        // All registers are read-only when APU is turned off
        if self.is_apu_enabled() {
            match address {
                0xFF10 => self.ch1.nr10 = value | 0b1000_0000,
                0xFF11 => self.ch1.nr11 = value,
                0xFF12 => self.ch1.nr12 = value,
                0xFF13 => self.ch1.nr13 = value,
                0xFF14 => self.ch1.nr14 = value | 0b0011_1000,

                0xFF16 => self.ch2.nr21 = value,
                0xFF17 => self.ch2.nr22 = value,
                0xFF18 => self.ch2.nr23 = value,
                0xFF19 => self.ch2.nr24 = value | 0b0011_1000,

                0xFF1A => self.ch3.nr30 = value | 0b0111_1111,
                0xFF1B => self.ch3.nr31 = value,
                0xFF1C => self.ch3.nr32 = value | 0b1001_1111,
                0xFF1D => self.ch3.nr33 = value,
                0xFF1E => self.ch3.nr34 = value | 0b0011_1000,

                0xFF20 => self.ch4.nr41 = value | 0b1100_0000,
                0xFF21 => self.ch4.nr42 = value,
                0xFF22 => self.ch4.nr43 = value,
                0xFF23 => self.ch4.nr44 = value | 0b0011_1111,

                0xFF24 => self.nr50 = value,
                0xFF25 => self.nr51 = value,
                _ => {}
            }
        }
    }
}

impl APU {
    pub fn tick(&mut self, div: u8) {
        // DIV-APU is increased when bit 4 of DIV (upper byte) goes from 1 to 0. (falling edge)
        if self.is_apu_enabled() && (div & 1 << 4) == 0 && self.div_bit == 1 {
            self.div_bit = (div & (1 << 4)) >> 4;
            self.div_apu += 1;


            // TODO:
            // envelope sweep every 8 ticks
            // sound length tick every 2 ticks
            // ch1 freq sweep every 4 ticks
        }
    }

    fn is_apu_enabled(&self) -> bool {
        self.nr52 & (1 << 7) != 0
    }
}
