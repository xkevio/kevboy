use eframe::epaint::Color32;

use crate::{
    cpu::interrupts::{self, Interrupt},
    ppu::ppu_regs::PPURegisters,
};

#[derive(PartialEq, Clone, Copy)]
enum Mode {
    Mode2 = 0b10,
    Mode3 = 0b11,
    VBlank = 0b1,
    HBlank = 0,
}

pub struct PPU {
    pub frame_buffer: [Color32; 256 * 256],
    current_line: Vec<u8>,
    regs: PPURegisters,
    cycles_passed: u16,
    current_mode: Mode,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            frame_buffer: [Color32::WHITE; 256 * 256],
            current_line: Vec::new(),
            regs: PPURegisters::default(),
            cycles_passed: 0,
            current_mode: Mode::Mode2,
        }
    }

    // should tick 4 times per m-cycle
    // 456 clocks per scanline
    // 80 (Mode2) - 172 (Mode3) - 204 (HBlank) - VBlank
    pub fn tick(&mut self, cycles_passed: u16, memory: &mut [u8]) {
        if self.regs.is_lcd_on() {
            self.cycles_passed += cycles_passed;

            if self.regs.ly_lyc() {
                self.regs.stat |= 0b100;

                if self.regs.stat & 0b1000000 != 0 {
                    interrupts::request_interrupt(memory, Interrupt::STAT);
                }
            } else {
                self.regs.stat &= !(0b100);
            }

            if self.cycles_passed >= 456 {
                self.regs.ly += 1;
                if self.regs.ly > 153 {
                    self.regs.ly = 0;
                }

                self.cycles_passed -= 456;
            }

            if self.regs.ly >= 144 && self.regs.ly <= 153 {
                self.change_mode(Mode::VBlank, memory);
            } else {
                match self.cycles_passed {
                    0..=80 => {
                        self.change_mode(Mode::Mode2, memory);
                        // scan oam and bg
                    }
                    81..=252 => {
                        self.change_mode(Mode::Mode3, memory);

                        // "draw" pixels into current line
                        if self.current_line.is_empty() {
                            self.current_line = self.get_bg_line(memory);
                        }
                    }
                    253..=456 => {
                        self.change_mode(Mode::HBlank, memory);

                        // add line to buffer
                        if self.cycles_passed >= 455 {
                            // println!("drawing line");
                            self.draw_line();
                        }
                    }
                    _ => panic!("More than 456 clocks have passed!"),
                }
            }
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF40 => self.regs.lcdc,
            0xFF41 => self.regs.stat,
            0xFF42 => self.regs.scy,
            0xFF43 => self.regs.scx,
            0xFF44 => self.regs.ly,
            0xFF45 => self.regs.lyc,
            0xFF47 => self.regs.bgp,
            0xFF48 => self.regs.opb0,
            0xFF49 => self.regs.opb1,
            0xFF4A => self.regs.wy,
            0xFF4B => self.regs.wx,
            _ => unreachable!(),
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0xFF40 => self.regs.lcdc = value,
            0xFF41 => self.regs.stat = value,
            0xFF42 => self.regs.scy = value,
            0xFF43 => self.regs.scx = value,
            0xFF44 => self.regs.ly = value,
            0xFF45 => self.regs.lyc = value,
            0xFF47 => self.regs.bgp = value,
            0xFF48 => self.regs.opb0 = value,
            0xFF49 => self.regs.opb1 = value,
            0xFF4A => self.regs.wy = value,
            0xFF4B => self.regs.wx = value,
            _ => unreachable!(),
        }
    }

    fn get_bg_line(&self, memory: &mut [u8]) -> Vec<u8> {
        // bg enable
        let mut bg_line: Vec<u8> = Vec::new();
        if self.regs.lcdc & 0b1 != 0 {
            let tile_map_area = if self.regs.lcdc & 0b1000 != 0 { // TODO
                0x9C00
            } else {
                0x9800
            };

            let unsigned_addressing = self.regs.lcdc & 0b10000 != 0;
            let test_start = tile_map_area + (((self.regs.ly / 8) as usize) * 0x20); 

            for index in test_start..=(test_start + 0x1F) {
                log::info!("index: {:#06X}, tile index: {}", index, memory[index] as usize  * 16);
                log::info!("ly: {}, unsigned addressing mode: {}", self.regs.ly, unsigned_addressing);
                let line_index = (memory[index] as usize) * 16;
                let ly_bytes = (self.regs.ly % 8) as usize;

                if unsigned_addressing {

                    log::info!("line index: {line_index}");
                    log::info!("bytes at: {:#06X}", 0x8000 + (line_index) + (2 * ly_bytes));
                    log::info!("bytes at: {:#06X}\n", 0x8000 + (line_index) + (2 * ly_bytes + 1));

                    let first_byte = memory[0x8000 + (line_index) + (2 * ly_bytes)];
                    let second_byte = memory[0x8000 + (line_index) + (2 * ly_bytes + 1)];

                    for i in (0..8).rev() {
                        let lsb = (first_byte & (1 << i)) >> i;
                        let msb = (second_byte & (1 << i)) >> i;

                        bg_line.push(msb << 1 | lsb);
                        // log::info!("{:#06b} | {:#06b}\n", msb << 1, lsb);
                    }
                } else {
                    let first_byte = if memory[index] <= 127 {
                        memory[0x9000 + (line_index * 8) + (2 * ly_bytes)]
                    } else {
                        memory[0x9000 - ((line_index * 8) + (2 * ly_bytes))]
                    };

                    let second_byte = if memory[index] <= 127 {
                        memory[0x9000 + (line_index * 8) + (2 * ly_bytes + 1)]
                    } else {
                        memory[0x9000 - ((line_index * 8) + (2 * ly_bytes + 1))]
                    };

                    for i in (0..8).rev() {
                        let lsb = (first_byte & (1 << i)) >> i;
                        let msb = (second_byte & (1 << i)) >> i;

                        bg_line.push((msb << 1) | lsb);
                    }
                }
            }

            bg_line
        } else {
            bg_line.fill(0b00);
            bg_line
        }
    }

    fn draw_line(&mut self) {
        fn bgp_color_from_value(value: u8) -> Color32 {
            match value {
                0b00 => Color32::WHITE,
                0b01 => Color32::LIGHT_GRAY,
                0b10 => Color32::GRAY,
                0b11 => Color32::BLACK,
                _ => unreachable!(),
            }
        }

        let y = self.regs.ly as usize;

        for i in 0..=255 {
            log::info!("{}", (y * 256 + i));
            self.frame_buffer[(y * 256 + i)] = match self.current_line[i as usize] {
                0b00 => bgp_color_from_value(self.regs.bgp & 0b11),
                0b01 => bgp_color_from_value((self.regs.bgp & 0b1100) >> 2),
                0b10 => bgp_color_from_value((self.regs.bgp & 0b110000) >> 4),
                0b11 => bgp_color_from_value((self.regs.bgp & 0b11000000) >> 6),
                _ => unreachable!(),
            }
        }

        self.current_line.clear();
    }

    fn change_mode(&mut self, to: Mode, memory: &mut [u8]) {
        if self.current_mode != to {
            match to {
                Mode::VBlank => {
                    interrupts::request_interrupt(memory, Interrupt::VBlank);
                    if self.regs.stat & 0b10000 != 0 {
                        interrupts::request_interrupt(memory, Interrupt::STAT);
                    }
                }
                Mode::HBlank => {
                    self.regs.stat &= !(0b11);

                    if self.regs.stat & 0b1000 != 0 {
                        interrupts::request_interrupt(memory, Interrupt::STAT);
                    }
                }
                Mode::Mode2 => {
                    if self.regs.stat & 0b100000 != 0 {
                        interrupts::request_interrupt(memory, Interrupt::STAT);
                    }
                }
                _ => {}
            }

            self.regs.stat |= to as u8;
            self.current_mode = to;
        }
    }
}
