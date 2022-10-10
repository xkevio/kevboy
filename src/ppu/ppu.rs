use crate::ppu::ppu_regs::PPURegisters;

enum Mode {
    Mode2,
    Mode3,
    VBlank,
    HBlank,
}

pub struct PPU {
    regs: PPURegisters,
    frame_buffer: [u8; 256 * 256],
    cycles_passed: u16,
    current_mode: Mode,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            regs: PPURegisters::default(),
            frame_buffer: [0; 256 * 256],
            cycles_passed: 0,
            current_mode: Mode::Mode2,
        }
    }

    // should tick 4 times per m-cycle
    // 456 clocks per scanline
    // 80 (Mode2) - 172 (Mode3) - 204 (HBlank) - VBlank
    pub fn tick(&mut self, cycles_passed: u16) {
        self.cycles_passed += cycles_passed;

        if self.regs.is_lcd_on() {
            if self.cycles_passed >= 456 {
                self.regs.ly += 1;
                if self.regs.ly > 153 {
                    self.regs.ly = 0;
                }

                self.cycles_passed -= 456;
            }

            if self.regs.ly >= 144 && self.regs.ly <= 153 {
                self.current_mode = Mode::VBlank;
            } else {
                match self.cycles_passed {
                    0..=80 => {
                        self.current_mode = Mode::Mode2;
                    }
                    81..=252 => {
                        self.current_mode = Mode::Mode3;
                    }
                    253..=456 => {
                        self.current_mode = Mode::HBlank;
                    }
                    _ => panic!("More than 456 clocks have passed!")
                }
            }
        }
    }

}