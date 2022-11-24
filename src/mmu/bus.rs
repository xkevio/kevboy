use crate::{
    cartridge::base_cartridge::Cartridge,
    cpu::interrupts::{Interrupt, InterruptHandler},
    input::joypad::Joypad,
    mmu::{mmio::MMIO, timer::Timers},
    ppu::ppu::PPU,
};

use super::serial::Serial;

pub struct Bus {
    pub cartridge: Cartridge,

    pub vram: [u8; 0x2000],
    pub wram: [u8; 0x2000],
    pub oam: [u8; 0xA0],

    pub joypad: Joypad,
    pub serial: Serial,
    pub timer: Timers,
    pub ppu: PPU, // lcdc, stat, scx, scy,...

    pub hram: [u8; 0xAF],
    pub interrupt_handler: InterruptHandler,

    disable_boot_rom: u8,
}

// ----------------------------
// MMIO trait for read/write (access via bus causes tick)
// ----------------------------

impl MMIO for Bus {
    fn read(&mut self, address: u16) -> u8 {
        self.tick(1);

        match address {
            0x0000..=0x7FFF => self.cartridge.read(address),
            0x8000..=0x9FFF => self.vram[address as usize - 0x8000],
            0xA000..=0xBFFF => self.cartridge.read(address),
            0xC000..=0xFDFF => self.wram[address as usize & 0x1FFF],
            0xFE00..=0xFE9F => self.oam[address as usize - 0xFE00],
            0xFEA0..=0xFEFF => 0xFF, // usage of this area not prohibited, may trigger oam corruption
            0xFF00..=0xFF7F => {
                match address {
                    0xFF00 => self.joypad.read(address),
                    0xFF01 | 0xFF02 => self.serial.read(address),
                    0xFF04..=0xFF07 => self.timer.read(address),
                    0xFF0F => self.interrupt_handler.intf,
                    // audio
                    0xFF24 => 0x00, // pokemon audio workaround
                    0xFF40..=0xFF4B => self.ppu.read(address),
                    0xFF50 => self.disable_boot_rom,
                    _ => 0xFF,
                }
            }
            0xFF80..=0xFFFE => self.hram[address as usize - 0xFF80],
            0xFFFF => self.interrupt_handler.inte,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        self.tick(1);

        match address {
            0x0000..=0x7FFF => self.cartridge.write(address, value),
            0x8000..=0x9FFF => self.vram[address as usize - 0x8000] = value,
            0xA000..=0xBFFF => self.cartridge.write(address, value),
            0xC000..=0xFDFF => self.wram[address as usize & 0x1FFF] = value,
            0xFE00..=0xFE9F => self.oam[address as usize - 0xFE00] = value,
            0xFEA0..=0xFEFF => {} // not usable area
            0xFF00..=0xFF7F => {
                match address {
                    0xFF00 => self.joypad.write(address, value),
                    0xFF01 | 0xFF02 => self.serial.write(address, value),
                    0xFF04..=0xFF07 => self.timer.write(address, value),
                    // audio
                    0xFF0F => self.interrupt_handler.intf = value,
                    0xFF40..=0xFF4B => self.ppu.write(address, value),
                    0xFF50 => self.disable_boot_rom = value,
                    _ => {}
                }
            }
            0xFF80..=0xFFFE => self.hram[address as usize - 0xFF80] = value,
            0xFFFF => self.interrupt_handler.inte = value,
        }
    }
}

// ----------------------------
// Normal impl for Bus
// ----------------------------

impl Bus {
    pub fn new() -> Self {
        Self {
            cartridge: Cartridge::default(),

            vram: [0xFF; 0x2000],
            wram: [0xFF; 0x2000],
            oam: [0xFF; 0xA0],

            joypad: Joypad::default(),
            serial: Serial::default(),
            timer: Timers::new(),
            ppu: PPU::new(),

            hram: [0xFF; 0xAF],
            interrupt_handler: InterruptHandler::default(),
            disable_boot_rom: 0x01,
        }
    }

    pub fn tick(&mut self, cycles_passed: u16) {
        if self.timer.irq {
            self.timer.irq = false;
            self.timer.tima = self.timer.tma;
            self.interrupt_handler.request_interrupt(Interrupt::Timer);
        }

        self.timer.tick(cycles_passed);
        self.serial.tick(&mut self.interrupt_handler, cycles_passed);

        // PPU ticks 4 times per M-cycle
        for _ in 0..(cycles_passed * 4) {
            self.ppu
                .tick(&self.vram, &self.oam, &mut self.interrupt_handler);
        }

        // TODO: dma is delayed one cycle -- write -> nothing -> DMA
        if self.ppu.is_dma_pending() {
            self.oam_dma_transfer();
        }
    }

    pub fn read_16(&mut self, address: u16) -> u16 {
        let lower_byte = self.read(address);
        let higher_byte = self.read(address + 1);

        (higher_byte as u16) << 8 | lower_byte as u16
    }

    pub fn write_16(&mut self, address: u16, value: u16) {
        let bytes = value.to_le_bytes();

        self.write(address, bytes[0]);
        self.write(address + 1, bytes[1]);
    }

    fn oam_dma_transfer(&mut self) {
        let source_start = (self.ppu.read(0xFF46) as u16) * 0x100;
        let source_end = source_start + 0x9F;

        self.ppu.reset_dma();

        for (dest_ind, addr) in (source_start..=source_end).enumerate() {
            self.oam[dest_ind] = self.read(addr);
        }
    }
}
