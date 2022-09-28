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

    pub fn step(&mut self) -> u8 {
        // TODO:
        // interrupt handling, ugly and should be changed (proof of concept)
        if self.cpu.ime {
            self.cpu.ime = false;

            let ie = self.bus.read_byte(0xFFFF);
            let if_flag = self.bus.read_byte(0xFF0F);

            for interrupt in interrupts::get_enabled_interrupts(ie) {
                if let Some(interr) = interrupt {
                    if interrupts::is_interrupt_requested(if_flag, &interr) {
                        self.cpu.registers.SP -= 1;
                        self.bus.write_byte(self.cpu.registers.SP, ((self.cpu.registers.PC) >> 8 & 0xFF00) as u8);
                        self.cpu.registers.SP -= 1;
                        self.bus.write_byte(self.cpu.registers.SP, self.cpu.registers.PC as u8);

                        self.bus.write_byte(0xFF0F, interrupts::reset_if(if_flag, &interr));
                        self.cpu.registers.PC = interr as u16;

                        self.cycle_count += 5;
                        return 5;
                    }
                }
            }
        }

        let cycles = self.cpu.tick(&mut self.bus);
        self.bus.tick(self.cycle_count);

        // print!("{}", self.bus.read_byte(0xFF01) as char);

        self.cycle_count += cycles as u64;

        cycles
    }
}