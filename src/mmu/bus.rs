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

    pub double_speed: bool,
    pub key1: u8,
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

        // Only matching on the top 4 bits seems to give better codegen and a
        // better jump table with less checks. (this function gets called a lot!)
        match (address & 0xF000) >> 12 {
            0x0..=0x7 => self.cartridge.read(address),
            0x8 | 0x9 => {
                let vbk = if self.ppu.cgb { self.vbk & 1 } else { 0 };
                self.vram[vbk as usize][address as usize - 0x8000]
            },
            0xA | 0xB => self.cartridge.read(address),
            0xC => self.wram[0][address as usize & 0x0FFF],
            0xD | 0xE => {
                // Echo RAM.
                if address > 0xDFFF && address < 0xF000 {
                    return self.wram[0][address as usize & 0x0FFF];
                }

                let wram_bank = if self.svbk & 0x07 == 0 { 1 } else { (self.svbk & 0x07) as usize };
                self.wram[if self.ppu.cgb { wram_bank } else { 1 }][address as usize & 0x0FFF]
            }
            0xF => {
                if address < 0xFE00 {
                    let wram_bank = if self.svbk & 0x07 == 0 { 1 } else { (self.svbk & 0x07) as usize };
                    return self.wram[if self.ppu.cgb { wram_bank } else { 1 }][address as usize & 0x0FFF];
                }

                match address & 0x0FFF {
                    0xE00..=0xE9F => self.oam[address as usize - 0xFE00],
                    0xEA0..=0xEFF => 0xFF, // usage of this area not prohibited, may trigger oam corruption
                    0xF00..=0xF7F => match address {
                        0xFF00 => self.joypad.read(address),
                        0xFF01 | 0xFF02 => self.serial.read(address),
                        0xFF04..=0xFF07 => self.timer.read(address),
                        0xFF0F => self.interrupt_handler.intf,
                        0xFF10..=0xFF3F => self.apu.read(address),
                        0xFF40..=0xFF4B | 0xFF68..=0xFF6B => self.ppu.read(address),
                        0xFF4D => self.key1,
                        0xFF4F => self.vbk,
                        0xFF50 => self.disable_boot_rom,
                        0xFF51..=0xFF55 => self.hdma.read(address),
                        0xFF70 => self.svbk,
                        _ => 0xFF,
                    },
                    0xF80..=0xFFE => self.hram[address as usize - 0xFF80],
                    0xFFF => self.interrupt_handler.inte,
                    _ => unreachable!()
                }
            }
            _ => unreachable!()
        }
    }

    #[rustfmt::skip]
    fn write(&mut self, address: u16, value: u8) {
        if self.ppu.get_dma_state() != DMATransferState::Transferring {
            self.tick(1);
        }

        match (address & 0xF000) >> 12 {
            0x0..=0x7 => self.cartridge.write(address, value),
            0x8 | 0x9 => {
                let vbk = if self.ppu.cgb { self.vbk & 1 } else { 0 };
                self.vram[vbk as usize][address as usize - 0x8000] = value;
            },
            0xA | 0xB => self.cartridge.write(address, value),
            0xC => self.wram[0][address as usize & 0x0FFF] = value,
            0xD | 0xE => {
                // Echo RAM.
                if address > 0xDFFF && address < 0xF000 {
                    self.wram[0][address as usize & 0x0FFF] = value;
                } else {
                    let wram_bank = if self.svbk & 0x07 == 0 { 1 } else { (self.svbk & 0x07) as usize };
                    self.wram[if self.ppu.cgb { wram_bank } else { 1 }][address as usize & 0x0FFF] = value;
                }
            }
            0xF => {
                if address < 0xFE00 {
                    let wram_bank = if self.svbk & 0x07 == 0 { 1 } else { (self.svbk & 0x07) as usize };
                    self.wram[if self.ppu.cgb { wram_bank } else { 1 }][address as usize & 0x0FFF] = value;
                    return;
                }

                match address & 0x0FFF {
                    0xE00..=0xE9F => self.oam[address as usize - 0xFE00] = value,
                    0xEA0..=0xEFF => {} // not usable area
                    0xF00..=0xF7F => match address {
                        0xFF00 => self.joypad.write(address, value),
                        0xFF01 | 0xFF02 => self.serial.write(address, value),
                        0xFF04..=0xFF07 => self.timer.write(address, value),
                        0xFF0F => self.interrupt_handler.intf = value | 0b1110_0000,
                        0xFF10..=0xFF3F => self.apu.write(address, value),
                        0xFF40..=0xFF4B | 0xFF68..=0xFF6B => {
                            self.ppu
                                .write_with_callback(address, value, || self.interrupt_handler.request_interrupt(Interrupt::STAT))
                        }
                        0xFF4D => self.key1 = (self.key1 & 0xFE) | (value & 1),
                        0xFF4F => if self.ppu.cgb { self.vbk = 0xFE | value },
                        0xFF51..=0xFF55 => {
                            self.hdma.write(address, value);
                            if address == 0xFF55 && self.ppu.cgb {
                                if (value & (1 << 7)) >> 7 == 0 && self.hdma.hdma_in_progress {
                                    self.hdma.terminate_transfer();
                                } else {
                                    self.vram_dma_transfer();
                                }
                            }
                        },
                        0xFF70 => if self.ppu.cgb { self.svbk = 0xF8 | (value & 0x07) },
                        _ => {}
                    },
                    0xF80..=0xFFE => self.hram[address as usize - 0xFF80] = value,
                    0xFFF => self.interrupt_handler.inte = value,
                    _ => unreachable!()
                }
            }
            _ => unreachable!()
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

            double_speed: false,
            key1: 0x7E,
        }
    }

    /// Ticks the bus in M-Cycles. Called every mem read/write
    /// and for extra cycles in certain instructions.
    ///
    /// Advances timer, serial and PPU for now.
    pub fn tick(&mut self, cycles_passed: u16) {
        let double_factor = if self.double_speed { 2 } else { 1 };

        let prev_tima = self.timer.tima;
        self.timer.tick(cycles_passed * double_factor);

        if self.timer.irq && prev_tima == 0 {
            self.timer.reload_tima();
            self.interrupt_handler.request_interrupt(Interrupt::Timer);
        }

        // Disable APU for testing and unthrottling!
        // TODO: Clock in T-cycles or M-cycles?
        // for _ in 0..((cycles_passed * 4) / double_factor) {
        //     self.apu.tick((self.timer.div >> 8) as u8);
        // }

        self.serial.tick(
            &mut self.interrupt_handler,
            cycles_passed * double_factor,
            self.timer.div,
        );

        // PPU ticks 4 times per M-cycle
        for _ in 0..((cycles_passed * 4) / double_factor) {
            if self.ppu.cgb {
                self.hdma.halted = true;
                for i in 0..0x10 {
                    self.hdma.bytes[i] = self.read(self.hdma.source() + i as u16);
                }
                self.hdma.halted = false;
            }

            self.ppu.tick(
                &mut self.vram,
                &self.oam,
                &mut self.interrupt_handler,
                &mut self.hdma,
                self.vbk & 1,
            );
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

    pub fn change_speed(&mut self) {
        let current_speed = (self.key1 & 0x80) >> 7;
        if current_speed == 0 {
            self.key1 = (self.key1 & 0x7F) | (1 << 7);
            self.double_speed = true;
        } else {
            self.key1 = (self.key1 & 0x7F) & !(1 << 7);
            self.double_speed = false;
        }

        self.key1 &= !1;
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
        let source = self.hdma.source();
        let dest = self.hdma.dest();
        let len = self.hdma.length();

        if self.hdma.is_gdma() && !self.hdma.hdma_in_progress {
            self.hdma.halted = true;

            for i in 0..len {
                self.vram[(self.vbk & 1) as usize][(dest + i) as usize] = self.read(source + i);
            }

            self.hdma.complete_transfer();
        } else {
            let hdma5 = self.hdma.read(0xFF55);
            self.hdma.write(0xFF55, hdma5 & 0x7F);
            self.hdma.hdma_in_progress = true;
        }
    }
}
