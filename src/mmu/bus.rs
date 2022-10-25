use crate::{
    cpu::{cpu::CPU, interrupts::InterruptHandler},
    mmu::timer::Timers,
    ppu::ppu::PPU,
};

pub struct Bus {
    pub memory: [u8; 0x10000], // one memory array not ideal
    pub timer: Timers,
    pub ppu: PPU,
    pub interrupt_handler: InterruptHandler,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            memory: [0xFF; 0x10000],
            timer: Timers::new(),
            ppu: PPU::new(),
            interrupt_handler: InterruptHandler::default(),
        }
    }

    pub fn load_rom_into_memory(&mut self, rom: &[u8]) {
        self.memory[..32768].copy_from_slice(rom);
        self.initialize_internal_registers();
    }

    pub fn tick(&mut self, cycles_passed: u16) {
        if self.timer.if_fired != 0 {
            self.memory[0xFF0F] |= self.timer.if_fired;
            self.interrupt_handler.intf |= self.timer.if_fired;
            self.timer.if_fired = 0;

            self.timer.tima = self.timer.tma;
        }

        self.timer.tick(cycles_passed);
        for _ in 0..4 {
            self.ppu
                .tick(cycles_passed, &mut self.memory, &mut self.interrupt_handler);
        }

        self.memory[0xFF04] = (self.timer.div >> 8) as u8;
        self.memory[0xFF05] = self.timer.tima;
    }

    pub fn read_16(&mut self, address: u16) -> u16 {
        let lower_byte = self.read_byte(address);
        let higher_byte = self.read_byte(address + 1);

        (higher_byte as u16) << 8 | lower_byte as u16
    }

    pub fn write_16(&mut self, address: u16, value: u16) {
        let bytes = value.to_le_bytes();

        self.write_byte(address, bytes[0]);
        self.write_byte(address + 1, bytes[1]);
    }

    pub fn read_byte(&mut self, address: u16) -> u8 {
        self.tick(1);

        match address {
            0xFF40..=0xFF4B => self.ppu.read_byte(address),
            0xFF0F => self.interrupt_handler.intf,
            0xFFFF => self.interrupt_handler.inte,
            _ => self.memory[address as usize],
        }
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        self.tick(1);
        match address {
            0x0000..=0x7FFF => {}
            0x8000..=0x9FFF => {
                // println!("VRAM access to {:#08X}", address);
                self.memory[address as usize] = byte;
            }
            0xA000..=0xBFFF => {
                // println!("External RAM access to {:#08X}", address);
                self.memory[address as usize] = byte;
            }
            0xC000..=0xDFFF => {
                // println!("WRAM access to {:#08X}", address);
                self.memory[address as usize] = byte;
            }
            0xE000..=0xFDFF => {} // println!("Echo RAM, ignore write"),
            0xFE00..=0xFE9F => {} // println!("OAM write, ignore"),
            0xFEA0..=0xFEFF => {} // println!("Not usable, usage of this area is prohibited"),
            0xFF00..=0xFF7F => {
                match address {
                    0xFF01 => eprint!("{}", byte as char), // SB output for blargg tests
                    0xFF04 => {
                        // DIV register: any write resets it to 0
                        self.memory[address as usize] = 0;
                        self.timer.reset_div();
                    }
                    0xFF05 => {
                        self.memory[address as usize] = byte;
                        self.timer.tima = byte;
                    }
                    0xFF06 => {
                        self.memory[address as usize] = byte;
                        self.timer.tma = byte;
                    }
                    0xFF07 => {
                        self.memory[address as usize] = byte;
                        self.timer.tac = byte;
                    }
                    0xFF40..=0xFF4B => {
                        self.ppu
                            .write_byte(address, byte, &mut self.interrupt_handler)
                    }
                    0xFF0F => {
                        self.interrupt_handler.intf = byte;
                        self.memory[address as usize] = byte;
                    }
                    _ => {
                        // println!("IO registers");
                        self.memory[address as usize] = byte;
                    }
                }
            }
            0xFF80..=0xFFFE => {
                // println!("HRAM access to {:#08X}", address);
                self.memory[address as usize] = byte;
            }
            0xFFFF => {
                // println!("Write to Interrupt Enable register (IE)");
                self.interrupt_handler.inte = byte;
                self.memory[address as usize] = byte;
            }
        }
    }

    pub fn handle_interrupts(&mut self, cpu: &mut CPU) -> bool {
        if cpu.ime {
            for interrupt in self
                .interrupt_handler
                .get_enabled_interrupts()
                .into_iter()
                .flatten()
            {
                if self.interrupt_handler.is_interrupt_requested(&interrupt) {
                    self.interrupt_handler.reset_if(&interrupt);
                    cpu.ime = false;
                    cpu.halt = false;

                    let pc_bytes = cpu.registers.PC.to_be_bytes();
                    self.tick(2); // 2 nop delay

                    cpu.registers.SP -= 1;
                    self.write_byte(cpu.registers.SP, pc_bytes[0]);

                    cpu.registers.SP -= 1;
                    self.write_byte(cpu.registers.SP, pc_bytes[1]);

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

    /// Initializes some internal memory-mapped registers
    /// to their values after booting on the DMG model.
    ///
    /// Only needed if no boot rom is used.
    fn initialize_internal_registers(&mut self) {
        self.memory[0xFF00] = 0xCF; // P1 / JOYP
        self.memory[0xFF07] = 0xF8; // TAC
        self.memory[0xFF4D] = 0xFF; // KEY1
        self.memory[0xFF50] = 0x01; // Disable BOOT ROM
        self.interrupt_handler.intf = 0xE1; // IF
        self.ppu
            .write_byte(0xFF40, 0x91, &mut self.interrupt_handler); // LCDC
        self.ppu
            .write_byte(0xFF41, 0x81, &mut self.interrupt_handler); // STAT
        self.ppu
            .write_byte(0xFF46, 0xFF, &mut self.interrupt_handler); // DMA
        self.ppu
            .write_byte(0xFF47, 0xFC, &mut self.interrupt_handler); // BGP
    }
}
