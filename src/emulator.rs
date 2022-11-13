use crate::cartridge::base_cartridge::{Cartridge, CartridgeType};
use crate::cartridge::mbc::mbc1::MBC1;
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

        // initialize cartridge, TODO: enum dispatch?
        let cartridge_type = match rom[0x0147] {
            0x00 => CartridgeType::NoMBC,
            0x01 | 0x02 | 0x03 => CartridgeType::MBC1(MBC1::default()),
            0x05 | 0x06 => CartridgeType::MBC2,
            0x0F..=0x13 => CartridgeType::MBC3,
            0x19..=0x1E => CartridgeType::MBC5,
            0x22 => CartridgeType::MBC7,
            _ => unimplemented!("Cartridge type not supported!"),
        };

        let rom_size_kb = 32 * (1 << rom[0x0148]);
        let ram_size_kb = match rom[0x0149] {
            0x00 => 0,
            0x02 => 8,
            0x03 => 32,
            0x04 => 128,
            0x05 => 64,
            _ => unimplemented!("RAM size not supported!"),
        };

        self.bus.cartridge = Cartridge::new(cartridge_type, rom_size_kb, ram_size_kb);
        self.rom = rom.to_vec();

        // always load the first 16kB into bank 0
        self.bus.load_rom_into_memory(&rom[..0x4000]);

        // load the second half into memory if it is a 32kB rom
        if cartridge_type == CartridgeType::NoMBC {
            self.bus.cartridge.rom_bank_x.copy_from_slice(&rom[0x4000..0x8000]);
        }

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
