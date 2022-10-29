use eframe::epaint::Color32;

use crate::{
    cpu::interrupts::{Interrupt, InterruptHandler},
    mmu::mmio::MMIO,
    ppu::{color_palette::*, ppu_regs::PPURegisters},
    LCD_HEIGHT, LCD_WIDTH,
};

#[derive(PartialEq, Clone, Copy)]
enum Mode {
    Mode2 = 0b10,
    Mode3 = 0b11,
    VBlank = 0b1,
    HBlank = 0,
}

#[allow(clippy::upper_case_acronyms)]
pub struct PPU {
    // LCD screen array, current viewport
    pub frame_buffer: [Color32; LCD_WIDTH * LCD_HEIGHT],
    current_line: Vec<u8>,

    regs: PPURegisters,
    cycles_passed: u16,

    current_mode: Mode,
    stat_block: bool,

    internal_window_line: u8,
}

impl MMIO for PPU {
    fn read(&self, address: u16) -> u8 {
        match address {
            0xFF40 => self.regs.lcdc,
            0xFF41 => self.regs.stat,
            0xFF42 => self.regs.scy,
            0xFF43 => self.regs.scx,
            0xFF44 => self.regs.ly,
            0xFF45 => self.regs.lyc,
            0xFF46 => self.regs.dma,
            0xFF47 => self.regs.bgp,
            0xFF48 => self.regs.opb0,
            0xFF49 => self.regs.opb1,
            0xFF4A => self.regs.wy,
            0xFF4B => self.regs.wx,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF40 => {
                self.regs.lcdc = value;

                if value & 0x80 == 0 {
                    self.turn_lcd_off();
                }
            }
            0xFF41 => self.regs.stat = ((1 << 7) | value) | (self.regs.stat & 0b11),
            0xFF42 => self.regs.scy = value,
            0xFF43 => self.regs.scx = value,
            0xFF44 => self.regs.ly = value,
            0xFF45 => self.regs.lyc = value,
            0xFF46 => self.regs.dma = value,
            0xFF47 => self.regs.bgp = value,
            0xFF48 => self.regs.opb0 = value,
            0xFF49 => self.regs.opb1 = value,
            0xFF4A => self.regs.wy = value,
            0xFF4B => self.regs.wx = value,
            _ => unreachable!(),
        }
    }
}

impl PPU {
    pub fn reset_window_line(&mut self) {
        self.internal_window_line = 0;
    }

    pub fn new() -> Self {
        Self {
            frame_buffer: [Color32::from_rgb(127, 134, 15); LCD_WIDTH * LCD_HEIGHT],
            current_line: Vec::new(),

            regs: PPURegisters::default(),
            cycles_passed: 0,

            current_mode: Mode::HBlank,
            stat_block: false,

            internal_window_line: 0,
        }
    }

    // should tick 4 times per m-cycle
    // 456 clocks per scanline
    // 80 (Mode2) - 172 (Mode3) - 204 (HBlank) - VBlank
    pub fn tick(
        &mut self,
        cycles_passed: u16,
        memory: &mut [u8],
        interrupt_handler: &mut InterruptHandler,
    ) {
        if self.regs.is_lcd_on() {
            if self.regs.ly_lyc() {
                self.regs.stat |= 0b100;
            } else {
                self.regs.stat &= !(0b100);
            }

            // ly=lyc check happens on the 4th cycle of the line
            if self.cycles_passed == 4 && self.check_stat_interrupt() {
                println!("request at ly: {}", self.regs.ly);
                interrupt_handler.request_interrupt(Interrupt::STAT);
            }

            if self.regs.ly < 144 {
                self.handle_line0_to_line143(memory, interrupt_handler);
            } else {
                if self.cycles_passed >= 456 {
                    self.regs.ly += 1;

                    if self.regs.ly > 153 {
                        self.regs.ly = 0;
                    }

                    self.cycles_passed -= 456;
                }
            }

            self.cycles_passed += cycles_passed;
        }
    }

    fn turn_lcd_off(&mut self) {
        self.regs.ly = 0;
        self.internal_window_line = 0;

        self.cycles_passed = 0;
        self.stat_block = false;

        self.change_mode(Mode::HBlank, None);
    }

    /// State machine that handles all lines and modes before VBlank
    fn handle_line0_to_line143(
        &mut self,
        memory: &mut [u8],
        interrupt_handler: &mut InterruptHandler,
    ) {
        match self.cycles_passed {
            0 => {
                self.change_mode(Mode::Mode2, Some(interrupt_handler));
                // scan oam for sprites
                // TODO
            }
            80 => {
                self.change_mode(Mode::Mode3, Some(interrupt_handler));

                // "draw" pixels into current line
                if self.current_line.is_empty() {
                    self.current_line = self.get_current_line(memory);
                }
            }
            252 => {
                // check at end of mode 3 technically
                if self.regs.is_window_visible()
                    && self.regs.ly >= self.regs.wy
                    && self.regs.is_window_enabled()
                {
                    self.internal_window_line += 1;
                }

                self.change_mode(Mode::HBlank, Some(interrupt_handler));

                // add line to buffer
                self.draw_current_line();
            }
            456 => {
                self.regs.ly += 1;
                self.stat_block = false;

                if self.regs.ly == 144 {
                    self.change_mode(Mode::VBlank, Some(interrupt_handler));
                } else {
                    self.change_mode(Mode::Mode2, Some(interrupt_handler));
                }

                self.cycles_passed -= 456;
            }
            _ => {}
        }
    }

    fn get_current_line(&mut self, memory: &mut [u8]) -> Vec<u8> {
        let mut current_line: Vec<u8> = Vec::new();

        // bg enable
        if self.regs.is_bg_enabled() {
            let unsigned_addressing = self.regs.lcdc & 0b10000 != 0;

            let bg_tile_map_area = if self.regs.lcdc & 0b1000 == 0 {
                0x9800
            } else {
                0x9C00
            };

            let adjusted_y = self.regs.ly + self.regs.scy;
            let tile_map_start = bg_tile_map_area + (((adjusted_y / 8) as usize) * 0x20);

            for index in tile_map_start..=(tile_map_start + 0x1F) {
                let tile_row = self.get_tile_row(memory, unsigned_addressing, index, adjusted_y);
                current_line.extend(tile_row);
            }

            // Apply SCX to the background layer
            current_line.rotate_left(self.regs.scx as usize);

            // Draw window over bg if enabled and visible
            if self.regs.is_window_enabled() {
                if self.regs.is_window_visible() && self.regs.ly >= self.regs.wy {
                    let win_tile_map_area = if self.regs.lcdc & 0x40 == 0 {
                        0x9800
                    } else {
                        0x9C00
                    };

                    let win_y = self.internal_window_line;
                    let tile_map_start = win_tile_map_area + (((win_y / 8) as usize) * 0x20);

                    println!("{:#06X}, ly: {}", tile_map_start, self.regs.ly);

                    for (j, index) in (tile_map_start..=(tile_map_start + 0x1F)).enumerate() {
                        let tile_row = self.get_tile_row(memory, unsigned_addressing, index, win_y);

                        for i in 0..8 {
                            if (((self.regs.wx - 7) as usize) + i + (j * 8)) < 256 {
                                current_line[((self.regs.wx - 7) as usize) + i + (j * 8)] = tile_row[i];
                            }
                        }
                    }
                }
            }

            current_line
        } else {
            vec![0b00; LCD_WIDTH]
        }
    }

    fn get_tile_row(
        &self,
        memory: &[u8],
        unsigned_addressing: bool,
        index: usize,
        y: u8,
    ) -> [u8; 8] {
        let mut current_line: [u8; 8] = [0; 8];

        let line_index = (memory[index] as usize) * 16;
        let ly_bytes = (y % 8) as usize;

        if unsigned_addressing {
            let first_byte = memory[0x8000 + line_index + (2 * ly_bytes)];
            let second_byte = memory[0x8000 + line_index + (2 * ly_bytes + 1)];

            for i in (0..8).rev() {
                let lsb = (first_byte & (1 << i)) >> i;
                let msb = (second_byte & (1 << i)) >> i;

                current_line[7 - i] = msb << 1 | lsb;
            }
        } else {
            let first_byte = if memory[index] <= 127 {
                memory[0x9000 + line_index + (2 * ly_bytes)]
            } else {
                let line_index = ((memory[index] as usize) % 128) * 16;
                memory[0x8800 + line_index + (2 * ly_bytes)]
            };

            let second_byte = if memory[index] <= 127 {
                memory[0x9000 + line_index + (2 * ly_bytes + 1)]
            } else {
                let line_index = ((memory[index] as usize) % 128) * 16;
                memory[0x8800 + line_index + (2 * ly_bytes + 1)]
            };

            for i in (0..8).rev() {
                let lsb = (first_byte & (1 << i)) >> i;
                let msb = (second_byte & (1 << i)) >> i;

                current_line[7 - i] = msb << 1 | lsb;
            }
        }

        current_line
    }

    fn draw_current_line(&mut self) {
        let y = self.regs.ly as usize;

        for i in 0..LCD_WIDTH {
            self.frame_buffer[y * LCD_WIDTH + i] = match self.current_line[i] {
                0b00 => bgp_color_from_value(self.regs.bgp & 0b11),
                0b01 => bgp_color_from_value((self.regs.bgp & 0b1100) >> 2),
                0b10 => bgp_color_from_value((self.regs.bgp & 0b110000) >> 4),
                0b11 => bgp_color_from_value((self.regs.bgp & 0b11000000) >> 6),
                _ => unreachable!(),
            }
        }

        self.current_line.clear();
    }

    // TODO: interrupt handler parameter
    fn change_mode(&mut self, to: Mode, interrupt_handler: Option<&mut InterruptHandler>) {
        match to {
            Mode::HBlank => self.regs.stat &= !(0b11),
            _ => {
                if to == Mode::VBlank {
                    let interrupt_handler = interrupt_handler.unwrap();
                    interrupt_handler.request_interrupt(Interrupt::VBlank);
                }
                self.regs.stat |= to as u8
            }
        }

        self.current_mode = to;
    }

    fn check_stat_interrupt(&mut self) -> bool {
        let prev_stat = self.stat_block;
        let current_stat = (self.regs.ly_lyc() && self.regs.stat & 0b1000000 != 0)
            || (self.current_mode == Mode::HBlank && self.regs.stat & 0b1000 != 0)
            || (self.current_mode == Mode::Mode2 && self.regs.stat & 0b100000 != 0)
            || (self.current_mode == Mode::VBlank && self.regs.stat & 0b10000 != 0);

        self.stat_block = current_stat;
        prev_stat != current_stat && current_stat
    }
}
