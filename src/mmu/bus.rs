use crate::{
    apu::apu::APU,
    cartridge::base_cartridge::Cartridge,
    cpu::interrupts::{Interrupt, InterruptHandler},
    input::joypad::Joypad,
    mmu::{mmio::MMIO, serial::Serial, timer::Timers},
    ppu::ppu::{DMATransferState, PPU},
};

use super::hdma_transfer::Hdma;

pub struct Bus {
    pub cartridge: Cartridge,

    pub vram: [[u8; 0x2000]; 2],
    pub wram: [[u8; 0x1000]; 8],
    pub oam: [u8; 0xA0],

    pub joypad: Joypad,
    pub serial: Serial,
    pub timer: Timers,
    pub ppu: PPU,

    pub apu: APU,

    pub hram: [u8; 0xAF],
    pub interrupt_handler: InterruptHandler,

    disable_boot_rom: u8,
    vbk: u8,
    svbk: u8,

    hdma: Hdma,
}

// ----------------------------
// MMIO trait for read/write (access via bus causes tick)
// ----------------------------

impl MMIO for Bus {
    #[rustfmt::skip]
    fn read(&mut self, address: u16) -> u8 {
        if self.ppu.get_dma_state() != DMATransferState::Transferring && !self.hdma.halted {
            self.tick(1);
        }

        match address {
            0x0000..=0x7FFF => self.cartridge.read(address),
            0x8000..=0x9FFF => self.vram[(self.vbk & 1) as usize][address as usize - 0x8000],
            0xA000..=0xBFFF => self.cartridge.read(address),
            0xC000..=0xCFFF => self.wram[0][address as usize & 0x0FFF],
            0xD000..=0xFDFF => {
                // Echo RAM.
                if address > 0xDFFF && address < 0xF000 {
                    return self.wram[0][address as usize & 0x0FFF];
                }

                let wram_bank = if self.svbk & 0x07 == 0 { 1 } else { (self.svbk & 0x07) as usize };
                self.wram[wram_bank][address as usize & 0x0FFF]
            }
            0xFE00..=0xFE9F => self.oam[address as usize - 0xFE00],
            0xFEA0..=0xFEFF => 0xFF, // usage of this area not prohibited, may trigger oam corruption
            0xFF00..=0xFF7F => match address {
                0xFF00 => self.joypad.read(address),
                0xFF01 | 0xFF02 => self.serial.read(address),
                0xFF04..=0xFF07 => self.timer.read(address),
                0xFF0F => self.interrupt_handler.intf,
                0xFF10..=0xFF3F => self.apu.read(address),
                0xFF40..=0xFF4B | 0xFF68..=0xFF6B => self.ppu.read(address),
                0xFF4F => self.vbk,
                0xFF50 => self.disable_boot_rom,
                0xFF51..=0xFF55 => self.hdma.read(address),
                0xFF70 => self.svbk,
                _ => 0xFF,
            },
            0xFF80..=0xFFFE => self.hram[address as usize - 0xFF80],
            0xFFFF => self.interrupt_handler.inte,
        }
    }

    #[rustfmt::skip]
    fn write(&mut self, address: u16, value: u8) {
        if self.ppu.get_dma_state() != DMATransferState::Transferring {
            self.tick(1);
        }

        match address {
            0x0000..=0x7FFF => self.cartridge.write(address, value),
            0x8000..=0x9FFF => self.vram[(self.vbk & 1) as usize][address as usize - 0x8000] = value,
            0xA000..=0xBFFF => self.cartridge.write(address, value),
            0xC000..=0xCFFF => self.wram[0][address as usize & 0x0FFF] = value,
            0xD000..=0xFDFF => {
                // Echo RAM.
                if address > 0xDFFF && address < 0xF000 {
                    self.wram[0][address as usize & 0x0FFF] = value;
                } else {
                    let wram_bank = if self.svbk & 0x07 == 0 { 1 } else { (self.svbk & 0x07) as usize };
                    self.wram[wram_bank][address as usize & 0x0FFF] = value;
                }
            }
            0xFE00..=0xFE9F => self.oam[address as usize - 0xFE00] = value,
            0xFEA0..=0xFEFF => {} // not usable area
            0xFF00..=0xFF7F => match address {
                0xFF00 => self.joypad.write(address, value),
                0xFF01 | 0xFF02 => self.serial.write(address, value),
                0xFF04..=0xFF07 => self.timer.write(address, value),
                0xFF0F => self.interrupt_handler.intf = value | 0b1110_0000,
                0xFF10..=0xFF3F => self.apu.write(address, value),
                0xFF40..=0xFF4B | 0xFF68..=0xFF6B => {
                    self.ppu
                        .write_with_callback(address, value, || self.interrupt_handler.request_interrupt(Interrupt::STAT))
                }
                0xFF4F => self.vbk = 0xFE | value,
                0xFF51..=0xFF55 => {
                    self.hdma.write(address, value);
                    if address == 0xFF55 {
                        self.hdma.halted = true;
                        self.vram_dma_transfer();
                    }
                },
                0xFF70 => self.svbk = 0xF8 | (value & 0x07),
                _ => {}
            },
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

            vram: [[0xFF; 0x2000]; 2],
            wram: [[0xFF; 0x1000]; 8],
            oam: [0xFF; 0xA0],

            joypad: Joypad::default(),
            serial: Serial::default(),
            timer: Timers::new(),
            ppu: PPU::new(),

            apu: APU::default(),

            hram: [0xFF; 0xAF],
            interrupt_handler: InterruptHandler::default(),
            disable_boot_rom: 0xFF, // not writable once unmapped
            vbk: 0xFF,
            svbk: 0xF8,

            hdma: Hdma::default(),
        }
    }

    /// Ticks the bus in M-Cycles. Called every mem read/write
    /// and for extra cycles in certain instructions.
    ///
    /// Advances timer, serial and PPU for now.
    pub fn tick(&mut self, cycles_passed: u16) {
        let prev_tima = self.timer.tima;
        self.timer.tick(cycles_passed);

        if self.timer.irq && prev_tima == 0 {
            self.timer.reload_tima();
            self.interrupt_handler.request_interrupt(Interrupt::Timer);
        }

        // TODO: Clock in T-cycles or M-cycles?
        for _ in 0..(cycles_passed * 4) {
            self.apu.tick((self.timer.div >> 8) as u8);
        }

        self.serial
            .tick(&mut self.interrupt_handler, cycles_passed, self.timer.div);

        // PPU ticks 4 times per M-cycle
        for _ in 0..(cycles_passed * 4) {
            self.ppu
                .tick(&self.vram, &self.oam, &mut self.interrupt_handler);
        }

        // DMA is delayed one cycle -- write -> nothing -> DMA
        match self.ppu.get_dma_state() {
            DMATransferState::Pending => self.ppu.set_dma_enable(),
            DMATransferState::Transferring => self.oam_dma_transfer(),
            DMATransferState::Disabled => {}
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

        for (dest_ind, addr) in (source_start..=source_end).enumerate() {
            self.oam[dest_ind] = self.read(addr);
        }

        self.ppu.reset_dma();
    }

    fn vram_dma_transfer(&mut self) {
        // TODO: HDMA
        if self.hdma.is_gdma() {
            let source = self.hdma.source();
            let dest = self.hdma.dest();
            let len = self.hdma.length();

            for i in 0..len {
                self.vram[(self.vbk & 1) as usize][(dest + i) as usize] = self.read(source + i);
            }

            self.hdma.complete_transfer();
        }
    }
}
