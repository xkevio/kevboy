use crate::{
    cpu::{cpu::CPU, interrupts::InterruptHandler},
    input::joypad::Joypad,
    mmu::{mmio::MMIO, timer::Timers},
    ppu::ppu::PPU, cartridge::base_cartridge::Cartridge,
};

pub struct Bus {
    pub cartridge: Cartridge,

    pub vram: [u8; 0x2000],
    pub wram: [u8; 0x2000],
    pub oam: [u8; 0xA0],

    pub joypad: Joypad,
    // serial...
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
            0xA000..=0xBFFF => {
                // read 0xFF when RAM is disabled
                self.cartridge.external_ram[address as usize - 0xA000]
            }
            0xC000..=0xFDFF => self.wram[address as usize & 0x1FFF],
            0xFE00..=0xFE9F => self.oam[address as usize - 0xFE00],
            0xFEA0..=0xFEFF => 0xFF, // usage of this area not prohibited, may trigger oam corruption
            0xFF00..=0xFF7F => {
                match address {
                    0xFF00 => self.joypad.read(address),
                    // serial
                    0xFF04..=0xFF07 => self.timer.read(address),
                    // audio
                    0xFF0F => self.interrupt_handler.intf,
                    0xFF40..=0xFF4B => self.ppu.read(address),
                    0xFF50 => self.disable_boot_rom,
                    _ => 0xFF,
                }
            }
            0xFF80..=0xFFFE => self.hram[address as usize - 0xFF80],
            0xFFFF => self.interrupt_handler.inte,
            // _ => unreachable!("Address out of scope: {:#06X}", address),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        self.tick(1);

        match address {
            0x0000..=0x7FFF => self.cartridge.write(address, value),
            0x8000..=0x9FFF => self.vram[address as usize - 0x8000] = value,
            0xA000..=0xBFFF => {
                // ignore writes when RAM is disabled and MBC0
                // self.external_ram[address as usize - 0xA000] = value
            }
            0xC000..=0xFDFF => self.wram[address as usize & 0x1FFF] = value,
            0xFE00..=0xFE9F => self.oam[address as usize - 0xFE00] = value,
            0xFEA0..=0xFEFF => {} // not usable area
            0xFF00..=0xFF7F => {
                match address {
                    0xFF00 => self.joypad.write(address, value),
                    // serial
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
            // _ => unreachable!("Write to invalid address: {:#06X}", address),
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
            timer: Timers::new(),
            ppu: PPU::new(),

            hram: [0xFF; 0xAF],
            interrupt_handler: InterruptHandler::default(),
            disable_boot_rom: 0,
        }
    }

    // loads 16kB into bank 0 and initializes hw registers
    pub fn load_rom_into_memory(&mut self, rom: &[u8]) {
        self.cartridge.rom_bank_0.copy_from_slice(rom);
        self.initialize_internal_registers();
    }

    pub fn tick(&mut self, cycles_passed: u16) {
        if self.timer.if_fired != 0 {
            self.interrupt_handler.intf |= self.timer.if_fired;
            self.timer.if_fired = 0;

            self.timer.tima = self.timer.tma;
        }

        self.timer.tick(cycles_passed);

        // PPU ticks 4 times per M-cycle
        for _ in 0..(cycles_passed * 4) {
            self.ppu
                .tick(&self.vram, &self.oam, &mut self.interrupt_handler);
        }

        // maybe delay?
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

    pub fn handle_interrupts(&mut self, cpu: &mut CPU) -> bool {
        if cpu.ime {
            for interrupt in self
                .interrupt_handler
                .get_enabled_interrupts()
                .into_iter()
                .flatten()
            {
                if self.interrupt_handler.is_interrupt_requested(interrupt) {
                    self.interrupt_handler.reset_if(interrupt);
                    cpu.ime = false;
                    cpu.halt = false;

                    let pc_bytes = cpu.registers.PC.to_be_bytes();
                    self.tick(2); // 2 nop delay

                    cpu.registers.SP -= 1;
                    self.write(cpu.registers.SP, pc_bytes[0]);

                    cpu.registers.SP -= 1;
                    self.write(cpu.registers.SP, pc_bytes[1]);

                    cpu.registers.PC = interrupt as u16;
                    self.tick(1);

                    return true;
                }
            }

            return false;
        } else {
            if self.interrupt_handler.inte & self.interrupt_handler.intf & 0x1F != 0 {
                cpu.halt = false;
            }

            return false;
        }
    }

    fn oam_dma_transfer(&mut self) {
        let source_start = (self.ppu.read(0xFF46) as u16) * 0x100;
        let source_end = source_start + 0x9F;

        self.ppu.reset_dma();

        for (dest_ind, addr) in (source_start..=source_end).enumerate() {
            self.oam[dest_ind] = self.read(addr);
        }
    }

    /// Initializes some internal memory-mapped registers
    /// to their values after booting on the DMG model.
    ///
    /// Only needed if no boot rom is used.
    fn initialize_internal_registers(&mut self) {
        self.joypad.write(0, 0xCF); // P1 / JOYP
        self.disable_boot_rom = 0x01; // 0xFF50
        self.interrupt_handler.intf = 0xE1; // IF

        self.ppu.write(0xFF40, 0x91); // LCDC
        self.ppu.write(0xFF41, 0x81); // STAT

        self.ppu.write(0xFF46, 0xFF); // DMA
        self.ppu.reset_dma();

        self.ppu.write(0xFF47, 0xFC); // BGP
    }
}
