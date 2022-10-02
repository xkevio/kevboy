use std::io::StdoutLock;

use crate::cpu::cpu::CPU;
use crate::cpu::interrupts;
use crate::mmu::bus::Bus;

pub struct Emulator {
    cpu: CPU,
    bus: Bus,
    cycle_count: u64,
}

impl Emulator {
    pub fn new(rom: &[u8]) -> Self {
        Self {
            cpu: CPU::new(),
            bus: Bus::new(rom),
            cycle_count: 0,
        }
    }

    pub fn step(&mut self, lock: &mut StdoutLock) -> u8 {
        // TODO:
        // interrupt handling, ugly and should be changed (proof of concept)
        let ie = self.bus.read_byte(0xFFFF);
        let if_flag = self.bus.read_byte(0xFF0F);

        if self.cpu.ime {
            for interrupt in interrupts::get_enabled_interrupts(ie) {
                if let Some(interr) = interrupt {
                    if interrupts::is_interrupt_requested(if_flag, &interr) {
                        let pc_bytes = self.cpu.registers.PC.to_be_bytes();

                        self.cpu.registers.SP -= 1;
                        self.bus.write_byte(self.cpu.registers.SP, pc_bytes[0]);

                        self.cpu.registers.SP -= 1;
                        self.bus.write_byte(self.cpu.registers.SP, pc_bytes[1]);

                        self.bus
                            .write_byte(0xFF0F, interrupts::reset_if(if_flag, &interr));

                        self.cpu.registers.PC = interr as u16;

                        self.cpu.ime = false;
                        self.cpu.halt = false;

                        self.cycle_count += 5;
                        return 5;
                    }
                }
            }
        } else {
            if ie & if_flag & 0x1F != 0 {
                self.cpu.halt = false;
            }
        }

        let cycles = self.cpu.tick(&mut self.bus, lock);
        self.cycle_count += cycles as u64; // this before or after bus/timer tick?

        self.bus.tick(self.cycle_count, cycles as u16);

        cycles
    }
}
