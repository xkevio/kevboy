use crate::cpu::cpu::CPU;
use crate::cpu::interrupts;
use crate::mmu::bus::Bus;

pub struct Emulator {
    pub cpu: CPU,
    pub bus: Bus,
    pub rom: Vec<u8>,
    pub cycle_count: u16,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            cpu: CPU::new(),
            bus: Bus::new(),
            rom: Vec::new(),
            cycle_count: 0,
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.reset();

        self.rom = rom.to_vec();
        self.bus.load_rom_into_memory(rom);
        self.cpu.registers.load_header_checksum(rom[0x014D]);
    }

    pub fn step(&mut self) -> u8 {
        // TODO: find out if to tick when reading IF or IE
        // interrupt handling, ugly and should be changed (proof of concept)
        let ie = self.bus.memory[0xFFFF];
        let if_flag = self.bus.memory[0xFF0F];

        if self.cpu.ime {
            for interrupt in interrupts::get_enabled_interrupts(ie).into_iter().flatten() {
                if interrupts::is_interrupt_requested(if_flag, &interrupt) {
                    let pc_bytes = self.cpu.registers.PC.to_be_bytes();
                    self.bus.tick(2); // 2 nop delay

                    self.cpu.registers.SP -= 1;
                    self.bus.write_byte(self.cpu.registers.SP, pc_bytes[0]);

                    self.cpu.registers.SP -= 1;
                    self.bus.write_byte(self.cpu.registers.SP, pc_bytes[1]);

                    self.bus
                        .write_byte(0xFF0F, interrupts::reset_if(if_flag, &interrupt));

                    self.cpu.registers.PC = interrupt as u16;

                    self.cpu.ime = false;
                    self.cpu.halt = false;

                    return 5;
                }
            }
        } else {
            if ie & if_flag & 0x1F != 0 {
                self.cpu.halt = false;
            }
        }

        self.cpu.tick(&mut self.bus)
    }

    fn reset(&mut self) {
        self.cpu = CPU::new();
        self.bus = Bus::new();
        self.rom = Vec::new();
        self.cycle_count = 0;
    }
}
