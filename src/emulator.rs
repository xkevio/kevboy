use crate::cpu::cpu::CPU;
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
        if self.bus.handle_interrupts(&mut self.cpu) {
            self.cycle_count += 5;
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
