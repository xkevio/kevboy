use crate::mmu::mmio::MMIO;
use rodio::{
    buffer::SamplesBuffer, queue::SourcesQueueOutput, OutputStream, OutputStreamHandle, Sink,
};

// WAVE DUTY CYCLES
const WAVE_DUTY_CYCLES: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 0],
];

/// Channel 1 produces square waves and uses both envelope and sweep
/// functionality. Uses the wave constants above to produce said signals.
struct ChannelOne {
    volume: u8,
    vol_timer: u8,

    sweep_timer: u8,

    len_counter: u8,
    duty_cycle: u8,
    freq_timer: u16,

    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
}

impl Default for ChannelOne {
    fn default() -> Self {
        Self {
            volume: 0,
            vol_timer: 0,

            sweep_timer: 0,

            len_counter: 0,
            duty_cycle: 0,
            freq_timer: 0,

            nr10: 0x80,
            nr11: 0xBF,
            nr12: 0xF3,
            nr13: 0xFF,
            nr14: 0xBF,
        }
    }
}

impl ChannelOne {
    /// Frequency timer ticks every T-cycle.
    pub fn duty_cycle(&mut self) {
        self.freq_timer -= 1;

        if self.freq_timer == 0 {
            let freq = ((self.nr14 & 0b111) as u16) << 8 | self.nr13 as u16;
            self.freq_timer = (2048 - freq) * 4;
            self.duty_cycle = (self.duty_cycle + 1) % 8;
        }
    }

    pub fn tick(&mut self, div_apu: u8, nr52: &mut u8) {
        if div_apu % 2 == 0 {
            if self.len_counter > 0 && self.nr14 & (1 << 6) != 0 {
                self.len_counter -= 1;

                // Turn channel off when length counter reaches zero
                if self.len_counter == 0 {
                    *nr52 &= !(1);
                }
            }
        }

        // Frequency sweep
        if div_apu % 4 == 0 {
            let sweep_pace = (self.nr10 & 0x70) >> 4;
            if sweep_pace != 0 {
                if self.sweep_timer > 0 {
                    self.sweep_timer -= 1;
                }

                if self.sweep_timer == 0 {
                    self.sweep_timer = sweep_pace;

                    let wave_length = ((self.nr14 & 0b111) as u16) << 8 | self.nr13 as u16;
                    let slope = self.nr10 & 0b111;

                    // 0 = increase, 1 = decrease
                    let new_wave_length = if self.nr10 & 0x08 == 0 {
                        wave_length + (wave_length / 2u16.pow(slope as u32))
                    } else {
                        wave_length - (wave_length / 2u16.pow(slope as u32))
                    };

                    // Turn channel off when wave length overflows 11 bit value in addition mode
                    if self.nr10 & 0x08 == 0 && new_wave_length > 0x7FF {
                        *nr52 &= !(1);
                    }

                    self.nr13 = new_wave_length as u8;
                    self.nr14 |= ((new_wave_length & 0x700) >> 8) as u8;
                }
            }
        }

        // Volume envelope
        if div_apu % 8 == 0 {
            if self.nr12 & 0b111 != 0 {
                if self.vol_timer > 0 {
                    self.vol_timer -= 1;
                }

                if self.vol_timer == 0 {
                    self.vol_timer = self.nr12 & 0b111;

                    if self.nr12 & 0x08 == 0 {
                        if self.volume > 0 {
                            self.volume -= 1
                        }
                    } else {
                        if self.volume < 0xF {
                            self.volume += 1
                        }
                    }
                }
            }
        }
    }

    /// Returns current sample of the square wave.
    ///
    /// Uses the DAC if it's on, otherwise returns zero.
    pub fn sample(&self) -> f32 {
        let sample = WAVE_DUTY_CYCLES[((self.nr11 & 0xC0) >> 6) as usize][self.duty_cycle as usize]
            * self.volume;

        if self.is_dac_on() {
            (sample as f32 / 7.5) - 1.0
        } else {
            0.0
        }
    }

    pub fn clear(&mut self) {
        self.nr10 = 0;
        self.nr11 = 0;
        self.nr12 = 0;
        self.nr13 = 0;
        self.nr14 = 0;
    }

    /// Turn channel on when setting bit 7 of NRx4 and DAC is on.
    /// Bit check is in the `write` method of APU.
    pub fn trigger(&mut self, nr52: &mut u8) {
        if self.len_counter == 0 {
            self.len_counter = 64;
        }

        self.volume = (self.nr12 & 0xF0) >> 4;
        self.vol_timer = self.nr12 & 0b111;
        self.freq_timer = self.nr12 as u16 & 0b111;
        self.sweep_timer = 0;

        // Only enable channel if DAC is on
        if self.is_dac_on() {
            *nr52 |= 1;
        }
    }

    fn is_dac_on(&self) -> bool {
        self.nr12 & 0xF8 != 0
    }
}

/// Channel 2 produces square waves and uses just the envelope functionality.
/// Uses the wave constants above to produce said signals.
struct ChannelTwo {
    volume: u8,
    vol_timer: u8,

    len_counter: u8,
    duty_cycle: u8,
    freq_timer: u16,

    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,
}

impl Default for ChannelTwo {
    fn default() -> Self {
        Self {
            volume: 0,
            vol_timer: 0,

            len_counter: 0,
            duty_cycle: 0,
            freq_timer: 0,

            nr21: 0x3F,
            nr22: 0x00,
            nr23: 0xFF,
            nr24: 0xBF,
        }
    }
}

impl ChannelTwo {
    /// Frequency timer ticks every T-cycle.
    pub fn duty_cycle(&mut self) {
        self.freq_timer -= 1;

        if self.freq_timer == 0 {
            let freq = ((self.nr24 & 0b111) as u16) << 8 | self.nr23 as u16;
            self.freq_timer = (2048 - freq) * 4;
            self.duty_cycle = (self.duty_cycle + 1) % 8;
        }
    }

    /// Ticks based on bit 4 of DIV, does length timing and volume envelope
    pub fn tick(&mut self, div_apu: u8, nr52: &mut u8) {
        // Length counter
        if div_apu % 2 == 0 {
            // Tick length counter
            if self.len_counter > 0 && self.nr24 & (1 << 6) != 0 {
                self.len_counter -= 1;

                // Turn channel off when length counter reaches zero
                if self.len_counter == 0 {
                    *nr52 &= !(1 << 1);
                }
            }
        }

        // Volume envelope
        if div_apu % 8 == 0 {
            if self.nr22 & 0b111 != 0 {
                if self.vol_timer > 0 {
                    self.vol_timer -= 1;
                }

                if self.vol_timer == 0 {
                    self.vol_timer = self.nr22 & 0b111;

                    if self.nr22 & 0x08 == 0 {
                        if self.volume > 0 {
                            self.volume -= 1
                        }
                    } else {
                        if self.volume < 0xF {
                            self.volume += 1
                        }
                    }
                }
            }
        }
    }

    /// Returns current sample of the square wave.
    ///
    /// Uses the DAC if it's on, otherwise returns zero.
    pub fn sample(&self) -> f32 {
        let sample = WAVE_DUTY_CYCLES[((self.nr21 & 0xC0) >> 6) as usize][self.duty_cycle as usize]
            * self.volume;

        if self.is_dac_on() {
            (sample as f32 / 7.5) - 1.0
        } else {
            0.0
        }
    }

    pub fn clear(&mut self) {
        self.nr21 = 0;
        self.nr22 = 0;
        self.nr23 = 0;
        self.nr24 = 0;
    }

    /// Turn channel on when setting bit 7 of NRx4 and DAC is on.
    /// Bit check is in the `write` method of APU.
    pub fn trigger(&mut self, nr52: &mut u8) {
        if self.len_counter == 0 {
            self.len_counter = 64;
        }

        self.volume = (self.nr22 & 0xF0) >> 4;
        self.vol_timer = self.nr22 & 0b111;
        self.freq_timer = self.nr22 as u16 & 0b111;

        // Only enable channel if DAC is on
        if self.is_dac_on() {
            *nr52 |= 1 << 1;
        }
    }

    fn is_dac_on(&self) -> bool {
        self.nr22 & 0xF8 != 0
    }
}

/// Channel 3 can produce custom waves from 4 bit samples based on Wave RAM.
struct ChannelThree {
    current_index: u8,
    len_counter: u16,
    freq_timer: u16,

    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,
}

impl Default for ChannelThree {
    fn default() -> Self {
        Self {
            current_index: 0,
            len_counter: 0,
            freq_timer: 0,

            nr30: 0x7F,
            nr31: 0xFF,
            nr32: 0x9F,
            nr33: 0xFF,
            nr34: 0xBF,
        }
    }
}

impl ChannelThree {
    /// Frequency timer ticks every T-cycle.
    pub fn duty_cycle(&mut self) {
        self.freq_timer -= 1;

        if self.freq_timer == 0 {
            let freq = ((self.nr34 & 0b111) as u16) << 8 | self.nr33 as u16;
            self.freq_timer = (2048 - freq) * 2;
            self.current_index = (self.current_index + 1) % 32;
        }
    }

    pub fn tick(&mut self, div_apu: u8, nr52: &mut u8) {
        if div_apu % 2 == 0 {
            if self.len_counter > 0 {
                self.len_counter -= 1;

                // Turn channel off when length counter reaches zero
                if self.len_counter == 0 && self.nr34 & (1 << 6) != 0 {
                    *nr52 &= !(1 << 2);
                }
            }
        }
    }

    /// Returns current sample of wave RAM.
    ///
    /// Uses the DAC if it's on, otherwise returns zero.
    pub fn sample(&self, wave_ram: &[u8]) -> f32 {
        let raw_sample = if self.current_index % 2 == 0 {
            (wave_ram[(self.current_index / 2) as usize] & 0xF0) >> 4
        } else {
            wave_ram[(self.current_index / 2) as usize] & 0x0F
        };

        let sample = match (self.nr32 & 0x60) >> 5 {
            0b00 => 0,
            0b01 => raw_sample,
            0b10 => raw_sample >> 1,
            0b11 => raw_sample >> 2,
            _ => unreachable!(),
        };

        if self.is_dac_on() {
            (sample as f32 / 7.5) - 1.0
        } else {
            0.0
        }
    }

    pub fn clear(&mut self) {
        self.nr30 = 0;
        self.nr31 = 0;
        self.nr32 = 0;
        self.nr33 = 0;
        self.nr34 = 0;
    }

    /// Turn channel on when setting bit 7 of NRx4 and DAC is on.
    /// Bit check is in the `write` method of APU.
    pub fn trigger(&mut self, nr52: &mut u8) {
        if self.len_counter == 0 {
            self.len_counter = 256;
        }

        // Quirk: ch3 starts at index 1, lower nibble of first byte
        self.current_index = 1;

        // Only enable channel if DAC is on
        if self.is_dac_on() {
            *nr52 |= 1 << 2;
        }
    }

    fn is_dac_on(&self) -> bool {
        self.nr30 & (1 << 7) != 0
    }
}

/// Channel 4 can produce pseudo random noise and also uses envelope.
struct ChannelFour {
    volume: u8,
    vol_timer: u8,

    len_counter: u8,
    lfsr: u16, // technically a 15 bit register
    freq_timer: u16,

    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,
}

impl Default for ChannelFour {
    fn default() -> Self {
        Self {
            volume: 0,
            vol_timer: 0,

            len_counter: 0,
            lfsr: 0,
            freq_timer: 0,

            nr41: 0xFF,
            nr42: 0x00,
            nr43: 0x00,
            nr44: 0xBF,
        }
    }
}

impl ChannelFour {
    /// Frequency timer ticks every T-cycle.
    pub fn duty_cycle(&mut self) {
        self.freq_timer -= 1;

        if self.freq_timer == 0 {
            let base_divisor = ((self.nr43 & 0b111) * 16).max(8) as u16;
            let clock_shift = (self.nr43 & 0xF0) >> 4;

            self.freq_timer = base_divisor << clock_shift;

            let xor_bit = (self.lfsr & 1) ^ ((self.lfsr & 2) >> 1);
            self.lfsr = (self.lfsr >> 1) & !(1 << 14);
            self.lfsr |= xor_bit << 14; // set bit 14 (15 bit register)

            // set bit 6 as well if LFSR width mode is set
            if self.nr43 & 0x08 != 0 {
                self.lfsr &= !(1 << 6);
                self.lfsr |= xor_bit << 6;
            }
        }
    }

    pub fn tick(&mut self, div_apu: u8, nr52: &mut u8) {
        if div_apu % 2 == 0 {
            if self.len_counter > 0 && self.nr44 & (1 << 6) != 0 {
                self.len_counter -= 1;

                // Turn channel off when length counter reaches zero
                if self.len_counter == 0 {
                    *nr52 &= !(1 << 3);
                }
            }
        }

        // Volume envelope
        if div_apu % 8 == 0 {
            if self.nr42 & 0b111 != 0 {
                if self.vol_timer > 0 {
                    self.vol_timer -= 1;
                }

                if self.vol_timer == 0 {
                    self.vol_timer = self.nr42 & 0b111;

                    if self.nr42 & 0x08 == 0 {
                        if self.volume > 0 {
                            self.volume -= 1
                        }
                    } else {
                        if self.volume < 0xF {
                            self.volume += 1
                        }
                    }
                }
            }
        }
    }

    /// Returns current sample of the square wave.
    ///
    /// Uses the DAC if it's on, otherwise returns zero.
    pub fn sample(&self) -> f32 {
        let sample = (!(self.lfsr as u8) & 1) * self.volume;

        if self.is_dac_on() {
            (sample as f32 / 7.5) - 1.0
        } else {
            0.0
        }
    }

    pub fn clear(&mut self) {
        self.nr41 = 0;
        self.nr42 = 0;
        self.nr43 = 0;
        self.nr44 = 0;
    }

    /// Turn channel on when setting bit 7 of NRx4 and DAC is on.
    /// Bit check is in the `write` method of APU.
    pub fn trigger(&mut self, nr52: &mut u8) {
        if self.len_counter == 0 {
            self.len_counter = 64;
        }

        self.lfsr = u16::MAX;
        self.volume = (self.nr42 & 0xF0) >> 4;
        self.vol_timer = self.nr42 & 0b111;

        let base_divisor = ((self.nr43 & 0b111) * 16).max(8) as u16;
        let clock_shift = (self.nr43 & 0xF0) >> 4;

        self.freq_timer = base_divisor << clock_shift;

        // Only enable channel if DAC is on
        if self.is_dac_on() {
            *nr52 |= 1 << 3;
        }
    }

    fn is_dac_on(&self) -> bool {
        self.nr42 & 0xF8 != 0
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
    /// Wave RAM holds 16 bytes of custom 4 bit samples for channel 3
    pub wave_ram: [u8; 0x10],
    /// Track T-cycles to play sound at correct time depending on sample rate
    internal_cycles: u8,

    /// Internal counter which increases based on falling edge of DIV
    div_apu: u8,
    /// Keep previous bit to detect falling edge
    div_bit: u8,

    ch1: ChannelOne,
    ch2: ChannelTwo,
    ch3: ChannelThree,
    ch4: ChannelFour,

    /// Global settings: master volume
    nr50: u8,
    /// Global settings: panning
    nr51: u8,
    /// Global settings: On/Off switch
    nr52: u8,

    /// Rodio frontend streams to play sound
    streams: (OutputStream, OutputStreamHandle),
    /// Queue to append samples to, never stops playing
    sink: (Sink, SourcesQueueOutput<f32>),
    init: bool,

    capacitor: f32,
}

impl Default for APU {
    fn default() -> Self {
        Self {
            wave_ram: [0xFF; 0x10],
            internal_cycles: 0,

            div_apu: 0,
            div_bit: 0,

            ch1: ChannelOne::default(),
            ch2: ChannelTwo::default(),
            ch3: ChannelThree::default(),
            ch4: ChannelFour::default(),

            nr50: 0x77,
            nr51: 0xF3,
            nr52: 0xF1,

            streams: OutputStream::try_default().unwrap(),
            sink: Sink::new_idle(),
            init: false,

            capacitor: 0.0,
        }
    }
}

impl MMIO for APU {
    // Here, we mask on reads and not writes since write-only bits are very present
    // and should always read back as 1. Plus, it makes clearing easier.
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0xFF10 => self.ch1.nr10 | 0x80,
            0xFF11 => self.ch1.nr11 | 0x3F,
            0xFF12 => self.ch1.nr12,
            0xFF13 => self.ch1.nr13 | 0xFF,
            0xFF14 => self.ch1.nr14 | 0xBF,

            0xFF16 => self.ch2.nr21 | 0x3F,
            0xFF17 => self.ch2.nr22,
            0xFF18 => self.ch2.nr23 | 0xFF,
            0xFF19 => self.ch2.nr24 | 0xBF,

            0xFF1A => self.ch3.nr30 | 0x7F,
            0xFF1B => self.ch3.nr31 | 0xFF,
            0xFF1C => self.ch3.nr32 | 0x9F,
            0xFF1D => self.ch3.nr33 | 0xFF,
            0xFF1E => self.ch3.nr34 | 0xBF,

            0xFF20 => self.ch4.nr41 | 0xFF,
            0xFF21 => self.ch4.nr42,
            0xFF22 => self.ch4.nr43,
            0xFF23 => self.ch4.nr44 | 0xBF,

            0xFF24 => self.nr50,
            0xFF25 => self.nr51,
            0xFF26 => self.nr52 | 0x70,

            0xFF30..=0xFF3F => self.wave_ram[(address - 0xFF30) as usize],
            _ => 0xFF,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        // NR52 is writable even with APU turned off
        if address == 0xFF26 {
            self.nr52 = (value & (1 << 7)) | (self.nr52 & 0x70);

            // Turning the APU off clears all registers besides NR52
            if !self.is_apu_enabled() {
                self.ch1.clear();
                self.ch2.clear();
                self.ch3.clear();
                self.ch4.clear();

                self.nr50 = 0;
                self.nr51 = 0;
            }
        }

        // Wave RAM is also readable and writable no matter the APU state
        if (0xFF30..=0xFF3F).contains(&address) {
            self.wave_ram[(address - 0xFF30) as usize] = value;
        }

        // All registers are read-only when APU is turned off
        if self.is_apu_enabled() {
            match address {
                0xFF10 => self.ch1.nr10 = value,
                0xFF11 => {
                    self.ch1.len_counter = 64 - (value & 0x3F);
                    self.ch1.nr11 = value;
                }
                0xFF12 => {
                    if value & 0xF8 == 0 {
                        // Turn DAC and ch1 off
                        self.nr52 &= !(1);
                    }
                    self.ch1.nr12 = value;
                }
                0xFF13 => self.ch1.nr13 = value,
                0xFF14 => {
                    if value & (1 << 7) != 0 {
                        self.ch1.trigger(&mut self.nr52);
                    }
                    self.ch1.nr14 = value;
                }

                0xFF16 => {
                    self.ch2.len_counter = 64 - (value & 0x3F);
                    self.ch2.nr21 = value;
                }
                0xFF17 => {
                    if value & 0xF8 == 0 {
                        // Turn DAC and ch2 off
                        self.nr52 &= !(1 << 1);
                    }
                    self.ch2.volume = (value & 0xF0) >> 4;
                    self.ch2.nr22 = value;
                }
                0xFF18 => self.ch2.nr23 = value,
                0xFF19 => {
                    if value & (1 << 7) != 0 {
                        self.ch2.trigger(&mut self.nr52);
                    }
                    self.ch2.nr24 = value;
                }

                0xFF1A => {
                    if value & (1 << 7) == 0 {
                        // Turn DAC and ch3 off
                        self.nr52 &= !(1 << 2);
                    }
                    self.ch3.nr30 = value;
                }
                0xFF1B => {
                    self.ch3.len_counter = 256 - value as u16;
                    self.ch3.nr31 = value;
                }
                0xFF1C => self.ch3.nr32 = value,
                0xFF1D => self.ch3.nr33 = value,
                0xFF1E => {
                    if value & (1 << 7) != 0 {
                        self.ch3.trigger(&mut self.nr52);
                    }
                    self.ch3.nr34 = value;
                }

                0xFF20 => {
                    self.ch4.len_counter = 64 - (value & 0x3F);
                    self.ch4.nr41 = value;
                }
                0xFF21 => {
                    if value & 0xF8 == 0 {
                        // Turn DAC and ch4 off
                        self.nr52 &= !(1 << 3);
                    }
                    self.ch4.nr42 = value;
                }
                0xFF22 => self.ch4.nr43 = value,
                0xFF23 => {
                    if value & (1 << 7) != 0 {
                        self.ch4.trigger(&mut self.nr52);
                    }
                    self.ch4.nr44 = value;
                }

                0xFF24 => self.nr50 = value,
                0xFF25 => self.nr51 = value,
                _ => {}
            }
        }
    }
}

impl APU {
    pub fn tick(&mut self, div: u8) {
        self.ch1.duty_cycle();
        self.ch2.duty_cycle();
        self.ch3.duty_cycle();
        self.ch4.duty_cycle();

        // DIV-APU is increased when bit 4 of DIV (upper byte) goes from 1 to 0. (falling edge)
        if self.is_apu_enabled() && (div & (1 << 4)) != 0x10 && self.div_bit == 1 {
            self.div_apu += 1;

            self.ch1.tick(self.div_apu, &mut self.nr52);
            self.ch2.tick(self.div_apu, &mut self.nr52);
            self.ch3.tick(self.div_apu, &mut self.nr52);
            self.ch4.tick(self.div_apu, &mut self.nr52);
        }

        // TODO: magic number (cpu freq / 44.1kHz)
        while self.internal_cycles >= 95 {
            self.internal_cycles -= 95;

            let ch1_sample = if self.is_ch1_enabled() { self.ch1.sample() } else { 0.0 };
            let ch2_sample = if self.is_ch2_enabled() { self.ch2.sample() } else { 0.0 };
            let ch3_sample = if self.is_ch3_enabled() { self.ch3.sample(&self.wave_ram) } else { 0.0 };
            let ch4_sample = if self.is_ch4_enabled() { self.ch4.sample() } else { 0.0 };

            // Sound panning via NR51
            let left_output = (self.nr51 & 0xF0) >> 4;
            let mut left_mix_sample = 0.0;

            let right_output = self.nr51 & 0x0F;
            let mut right_mix_sample = 0.0;

            for (i, sample) in [ch1_sample, ch2_sample, ch3_sample, ch4_sample]
                .iter()
                .enumerate()
            {
                left_mix_sample += if left_output & (1 << i) != 0 {
                    *sample
                } else {
                    0.0
                };

                right_mix_sample += if right_output & (1 << i) != 0 {
                    *sample
                } else {
                    0.0
                };
            }

            left_mix_sample /= 4.0;
            right_mix_sample /= 4.0;

            // Play silence when channel is disabled, otherwise mix DAC sample for left and right channel
            let left_sample = left_mix_sample.signum()
                * (left_mix_sample.abs() * (1.0 / (((self.nr50 & 0x70) >> 4) as f32 + 1.0)));
            let right_sample = right_mix_sample.signum()
                * (right_mix_sample.abs() * (1.0 / ((self.nr50 & 0b111) as f32 + 1.0)));

            let ls = self.high_pass(left_sample);
            let rs = self.high_pass(right_sample);

            self.sink.0.append(SamplesBuffer::new(2, 44100, [ls, rs]));
        }

        // TODO: init Sink directly with OutputStreamHandle
        if !self.init {
            self.sink.0 = Sink::try_new(&self.streams.1).unwrap();
            self.init = true;
        }

        self.internal_cycles += 1;
        self.div_bit = (div & (1 << 4)) >> 4;
    }

    /// Checks if the APU is enabled by checking bit 7 of NR52.
    ///
    /// - If **on**, channels get ticked and internal values updated
    /// - If **off**, only duty cycles and internal cycles get updated
    fn is_apu_enabled(&self) -> bool {
        self.nr52 & (1 << 7) != 0
    }

    /// High-Pass filter capacitor which slowly removes DC offset.
    ///
    /// Runs after DAC conversion so that a digital volume of 0 which gets converted to -1
    /// slowly gets removed and turned back to silence.
    ///
    /// Charge factor: 0.999958^(4MHz / sample rate)
    fn high_pass(&mut self, in_sample: f32) -> f32 {
        let out = in_sample - self.capacitor;
        self.capacitor = in_sample - out * 0.996;

        return out;
    }

    // -------- CHANNEL STATUS --------
    fn is_ch1_enabled(&self) -> bool {
        self.nr52 & (1) != 0
    }

    fn is_ch2_enabled(&self) -> bool {
        self.nr52 & (1 << 1) != 0
    }

    fn is_ch3_enabled(&self) -> bool {
        self.nr52 & (1 << 2) != 0
    }

    fn is_ch4_enabled(&self) -> bool {
        self.nr52 & (1 << 3) != 0
    }
}
