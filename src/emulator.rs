use crate::cartridge::base_cartridge::{Cartridge, CartridgeType};
use crate::cartridge::mbc::mbc1::MBC1;
use crate::cartridge::mbc::mbc2::MBC2;
use crate::cartridge::mbc::mbc3::MBC3;
use crate::cartridge::mbc::mbc5::MBC5;
use crate::cartridge::mbc::no_mbc::NoMBC;
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
        let rom_size_kb = 32 * (1 << rom[0x0148]);
        let ram_size_kb = match rom[0x0149] {
            0x00 => 0,
            0x02 => 8,
            0x03 => 32,
            0x04 => 128,
            0x05 => 64,
            _ => unimplemented!("RAM size not supported!"),
        };

        let cartridge_type = match rom[0x0147] {
            0x00 => CartridgeType::NoMBC(NoMBC::new(rom)),
            0x01 | 0x02 | 0x03 => CartridgeType::MBC1(MBC1::new(rom, rom_size_kb, ram_size_kb)),
            0x05 | 0x06 => CartridgeType::MBC2(MBC2::new(rom)),
            0x0F..=0x13 => CartridgeType::MBC3(MBC3::new(rom)),
            0x19..=0x1E => CartridgeType::MBC5(MBC5::new(rom, rom_size_kb, ram_size_kb)),
            0x22 => CartridgeType::MBC7,
            _ => unimplemented!("Cartridge type not supported!"),
        };

        let title = std::str::from_utf8(&rom[0x0134..=0x0143])
            .or_else(|_| std::str::from_utf8(&rom[0x0134..=0x0142]))
            .or_else(|_| std::str::from_utf8(&rom[0x0134..=0x013E]))
            .unwrap();
        println!("{title}");
        println!("MBC: {:#06X}", rom[0x0147]);

        self.bus.cartridge = Cartridge::new(cartridge_type, title);
        self.rom = rom.to_vec(); // TODO: redundant?
        self.cpu.registers.load_header_checksum(rom[0x014D]);
    }

    pub fn step(&mut self) -> u8 {
        if self.cpu.handle_interrupts(&mut self.bus) {
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
