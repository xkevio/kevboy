use crate::cpu::interrupts::{Interrupt, InterruptHandler};
use crate::mmu::mmio::MMIO;

pub struct Serial {
    sb: u8,
    sc: u8,

    counter: u8,
    and_result_falling_edge: bool,
}

impl Default for Serial {
    fn default() -> Self {
        Self {
            sb: 0x00,
            sc: 0x7E,

            counter: 1,
            and_result_falling_edge: false,
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
    pub fn tick(&mut self, interrupt_handler: &mut InterruptHandler, cycles_passed: u16, div: u16) {
        for _ in 0..(cycles_passed * 4) {
            if self.and_result_falling_edge && !(((div & (1 << 8)) != 0) & self.is_internal_clock())
            {
                if self.counter <= 8 {
                    self.sb = (self.sb << self.counter) | (0xFF >> (8 - self.counter));
                    self.counter += 1;
                }

                if self.counter > 8 {
                    self.counter = 1;
                    self.sc = 0x01;

                    interrupt_handler.request_interrupt(Interrupt::Serial);
                }
            }

            self.and_result_falling_edge = ((div & (1 << 8)) != 0) & self.is_internal_clock();
        }
    }

    // bit 0 is internal or external clock -- external clock is effectively disable for emulation
    fn is_internal_clock(&self) -> bool {
        self.sc & 1 != 0
    }
}
