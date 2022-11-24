use crate::cpu::interrupts::{Interrupt, InterruptHandler};

use super::mmio::MMIO;

/// It takes 128 m-cycles to receive / transfer one bit (8192Hz)
const DMG_CLOCK_FREQ_CYCLES: u16 = 128;

pub struct Serial {
    sb: u8,
    sc: u8,

    counter: u8,
    cycles_passed: u16,
}

impl Default for Serial {
    fn default() -> Self {
        Self {
            sb: 0x00,
            sc: 0x7E,

            counter: 1,
            cycles_passed: 0,
        }
    }
}

impl MMIO for Serial {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0xFF01 => self.sb,
            0xFF02 => self.sc,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF01 => self.sb = value,
            0xFF02 => self.sc = value,
            _ => unreachable!(),
        }
    }
}

impl Serial {
    pub fn tick(&mut self, interrupt_handler: &mut InterruptHandler, cycles_passed: u16) {
        self.cycles_passed += cycles_passed;

        while self.cycles_passed >= DMG_CLOCK_FREQ_CYCLES {
            // always assume internal clock (DMG) and 0xFF as receiving byte for emulation
            if self.is_transfer_requested() && self.counter <= 8 {
                self.sb = (self.sb << self.counter) | (0xFF >> (8 - self.counter));
                self.counter += 1;
            }

            if self.counter > 8 {
                self.counter = 1;
                self.sc = 0x01;

                interrupt_handler.request_interrupt(Interrupt::Serial);
            }

            self.cycles_passed -= DMG_CLOCK_FREQ_CYCLES;
        }
    }

    fn is_transfer_requested(&self) -> bool {
        self.sc & (1 << 7) != 0
    }
}
