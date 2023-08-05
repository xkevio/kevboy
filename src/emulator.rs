use crate::cartridge::base_cartridge::{Cartridge, CartridgeType};
use crate::cartridge::mbc::mbc1::MBC1;
use crate::cartridge::mbc::mbc2::MBC2;
use crate::cartridge::mbc::mbc3::MBC3;
use crate::cartridge::mbc::mbc5::MBC5;
use crate::cartridge::mbc::no_mbc::NoMBC;
use crate::cpu::cpu::CPU;
use crate::cpu::registers::Registers;
use crate::mmu::bus::Bus;

pub struct Emulator {
    pub cpu: CPU,
    pub bus: Bus,
    pub rom: Vec<u8>,
    pub cycle_count: u16,
    cgb: bool,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            cpu: CPU::new(),
            bus: Bus::new(),
            rom: Vec::new(),
            cycle_count: 0,
            cgb: false,
        }
    }

    /// Load ROM and dispatch correct MBC based on header bytes.
    ///
    /// Read out title, RAM and ROM size and set flags based on header
    /// checksum. Initializes `Cartridge` for the Bus.
    pub fn load_rom(&mut self, rom: &[u8]) {
        self.reset();

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
            _ => unimplemented!("Cartridge type not supported!"),
        };

        let title = std::str::from_utf8(&rom[0x0134..=0x0143])
            .or_else(|_| std::str::from_utf8(&rom[0x0134..=0x0142]))
            .or_else(|_| std::str::from_utf8(&rom[0x0134..=0x013E]))
            .unwrap();

        self.cgb = rom[0x0143] == 0x80 || rom[0x0143] == 0xC0;
        self.bus.cartridge = Cartridge::new(cartridge_type, title);
        self.rom = rom.to_vec(); // TODO: redundant?

        if self.cgb {
            self.cpu.cgb = true;
            self.cpu.registers = Registers::new_cgb();
            self.bus.ppu.enable_cgb();
        } else {
            self.cpu.registers = Registers::new_dmg(rom[0x014D]);
        }
    }

    /// Step emulator by ticking CPU, advancing it one instruction and returning
    /// the cycles it took.
    ///
    /// `Bus` is passed for sub-instruction level accuracy so that the bus
    /// and its components can tick during instructions.
    ///
    /// Handles interrupts and returns the appropriate amount of cycles if one occured.
    pub fn step(&mut self) -> u8 {
        if self.cpu.handle_interrupts(&mut self.bus) {
            self.cycle_count += 5;
        }

        self.cpu.tick(&mut self.bus)
    }

    // ------------ CARTRIDGE INFO FOR DISPLAY ---------------
    pub fn get_full_mbc_title(&self) -> Option<&str> {
        if self.rom.is_empty() {
            return None;
        }

        match self.rom[0x0147] {
            0x00 => Some("ROM ONLY"),
            0x01 => Some("MBC1"),
            0x02 => Some("MBC1+RAM"),
            0x03 => Some("MBC1+RAM+BATTERY"),
            0x05 => Some("MBC2"),
            0x06 => Some("MBC2+BATTERY"),
            0x0F => Some("MBC3+TIMER+BATTERY"),
            0x10 => Some("MBC3+TIMER+RAM+BATTERY"),
            0x11 => Some("MBC3"),
            0x12 => Some("MBC3+RAM"),
            0x13 => Some("MBC3+RAM+BATTERY"),
            0x19 => Some("MBC5"),
            0x1A => Some("MBC5+RAM"),
            0x1B => Some("MBC5+RAM+BATTERY"),
            0x1C => Some("MBC5+RUMBLE"),
            0x1D => Some("MBC5+RUMBLE+RAM"),
            0x1E => Some("MBC5+RUMBLE+RAM+BATTERY"),
            0x20 => Some("MBC6"),
            0x22 => Some("MBC7+SENSOR+RUMBLE+RAM+BATTERY"),
            _ => None,
        }
    }

    pub fn get_destination_code(&self) -> Option<&str> {
        if self.rom.is_empty() {
            return None;
        }

        match self.rom[0x14A] {
            0x00 => Some("Japan (and possibly overseas)"),
            0x01 => Some("Overseas only"),
            _ => None,
        }
    }

    pub fn get_rom_size(&self) -> Option<usize> {
        if self.rom.is_empty() {
            return None;
        }

        Some(32 * (1 << self.rom[0x0148]))
    }

    pub fn get_ram_size(&self) -> Option<u8> {
        if self.rom.is_empty() {
            return None;
        }

        match self.rom[0x0149] {
            0x00 => Some(0),
            0x02 => Some(8),
            0x03 => Some(32),
            0x04 => Some(128),
            0x05 => Some(64),
            _ => unimplemented!("RAM size not supported!"),
        }
    }
    // ------------ CARTRIDGE INFO FOR DISPLAY ---------------

    pub fn reset(&mut self) {
        self.cpu = CPU::new();
        self.bus = Bus::new();
        self.rom = Vec::new();
        self.cycle_count = 0;
        self.cgb = false;
    }

    pub fn is_cgb(&self) -> bool {
        self.cgb
    }
}
