use crate::mmu::timer::Timers;

pub struct Bus {
    memory: [u8; 0xFFFF + 1],
    pub timer: Timers,
}

impl Bus {
    pub fn new(rom: &[u8]) -> Self {
        let mut memory = [0u8; 0xFFFF + 1];
        memory[..32768].copy_from_slice(rom);

        Self {
            memory,
            timer: Timers::new(),
        }
    }

    pub fn tick(&mut self, m_cycles: u64) {
        self.timer.tick(m_cycles);

        self.memory[0xFF04] = (self.timer.div >> 8) as u8;
        self.memory[0xFF05] = self.timer.tima;

        self.memory[0xFF0F] |= self.timer.if_fired;
        self.timer.if_fired = 0;
    }

    pub fn read_16(&self, address: u16) -> u16 {
        let lower_byte = self.read_byte(address);
        let higher_byte = self.read_byte(address + 1);

        (higher_byte as u16) << 8 | lower_byte as u16
    }

    pub fn write_16(&mut self, address: u16, value: u16) {
        let bytes = value.to_le_bytes();

        self.write_byte(address, bytes[0]);
        self.write_byte(address + 1, bytes[1]);
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x7FFF => println!("write to Read-Only-Memory, ignore for now"),
            0x8000..=0x9FFF => {
                println!("VRAM access to {:#08X}", address);
                self.memory[address as usize] = byte;
            }
            0xA000..=0xBFFF => {
                println!("External RAM access to {:#08X}", address);
                self.memory[address as usize] = byte;
            }
            0xC000..=0xDFFF => {
                println!("WRAM access to {:#08X}", address);
                self.memory[address as usize] = byte;
            }
            0xE000..=0xFDFF => println!("Echo RAM, ignore write"),
            0xFE00..=0xFE9F => println!("OAM write, ignore"),
            0xFEA0..=0xFEFF => println!("Not usable, usage of this area is prohibited"),
            0xFF00..=0xFF7F => {
                match address {
                    0xFF01 => print!("{}", byte as char), // SB output for blargg tests
                    0xFF04 => {
                        // DIV register: any write resets it to 0
                        self.memory[address as usize] = 0;
                        self.timer.div = 0;
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
                    _ => {    
                        println!("IO registers");
                        self.memory[address as usize] = byte;
                    }
                }
            }
            0xFF80..=0xFFFE => {
                println!("HRAM access to {:#08X}", address);
                self.memory[address as usize] = byte;
            }
            0xFFFF => {
                println!("Write to Interrupt Enable register (IE)");
                self.memory[address as usize] = byte;
            }
        }
    }
}