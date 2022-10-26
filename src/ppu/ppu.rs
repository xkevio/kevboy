use eframe::epaint::Color32;

use crate::{
    cpu::interrupts::{Interrupt, InterruptHandler},
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
    // complete background map with all tiles
    // pub bg_map: [Color32; 256 * 256],
    pub test_map: [Color32; LCD_WIDTH * LCD_HEIGHT],
    current_line: Vec<u8>,

    regs: PPURegisters,
    cycles_passed: u16,

    current_mode: Mode,
    stat_block: bool,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            // bg_map: [Color32::from_rgb(127, 134, 15); 256 * 256],
            test_map: [Color32::from_rgb(127, 134, 15); LCD_WIDTH * LCD_HEIGHT],
            current_line: Vec::new(),
            regs: PPURegisters::default(),
            cycles_passed: 0,
            current_mode: Mode::HBlank,
            stat_block: false,
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
            self.cycles_passed += cycles_passed;

            if self.cycles_passed == 4 && self.check_stat_interrupt() {
                // println!("stat interrupt at ly: {:#04X}, lcdc: {:#010b}, ({}, {})", self.regs.ly, self.regs.lcdc, self.regs.scx, self.regs.scy);
                interrupt_handler.request_interrupt(Interrupt::STAT);
            }

            if self.regs.ly_lyc() {
                self.regs.stat |= 0b100;
            } else {
                self.regs.stat &= !(0b100);
            }

            if self.cycles_passed >= 456 {
                self.regs.ly += 1;
                self.stat_block = false;

                if self.regs.ly > 153 {
                    self.regs.ly = 0;
                }

                self.cycles_passed -= 456;
            }

            if self.regs.ly >= 144 && self.regs.ly <= 153 {
                self.change_mode(Mode::VBlank, interrupt_handler);
            } else {
                match self.cycles_passed {
                    0..=80 => {
                        self.change_mode(Mode::Mode2, interrupt_handler);
                        // scan oam and bg
                    }
                    81..=252 => {
                        self.change_mode(Mode::Mode3, interrupt_handler);

                        // "draw" pixels into current line
                        if self.current_line.is_empty() {
                            self.current_line = self.get_current_line(memory);
                        }
                    }
                    253..=456 => {
                        self.change_mode(Mode::HBlank, interrupt_handler);

                        // add line to buffer
                        if self.cycles_passed >= 455 {
                            self.draw_current_line();
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
            0xFF46 => self.regs.dma,
            0xFF47 => self.regs.bgp,
            0xFF48 => self.regs.opb0,
            0xFF49 => self.regs.opb1,
            0xFF4A => self.regs.wy,
            0xFF4B => self.regs.wx,
            _ => unreachable!(),
        }
    }

    pub fn write_byte(
        &mut self,
        address: u16,
        value: u8,
        interrupt_handler: &mut InterruptHandler,
    ) {
        // println!("Write to {:#06X} = {:#06X}", address, value);

        match address {
            0xFF40 => {
                self.regs.lcdc = value;

                log::info!("Write to LCDC");

                if value & 0x80 == 0 {
                    self.regs.ly = 0;
                    self.cycles_passed = 0;
                    self.stat_block = false;

                    log::info!("lcd turn off: stat {:#010b}", self.regs.stat);
                    self.change_mode(Mode::HBlank, interrupt_handler);
                }
            }
            0xFF41 => self.regs.stat = ((1 << 7) | value) | (self.regs.stat & 0b11),
            0xFF42 => self.regs.scy = value,
            0xFF43 => self.regs.scx = value,
            0xFF44 => self.regs.ly = value,
            0xFF45 => {
                self.regs.lyc = value;

                // if self.check_stat_interrupt() {
                //     println!("LYC write: STAT irq at ly: {}", self.regs.ly);
                //     interrupt_handler.request_interrupt(Interrupt::STAT);
                // }
            }
            0xFF46 => self.regs.dma = value,
            0xFF47 => self.regs.bgp = value,
            0xFF48 => self.regs.opb0 = value,
            0xFF49 => self.regs.opb1 = value,
            0xFF4A => self.regs.wy = value,
            0xFF4B => self.regs.wx = value,
            _ => unreachable!(),
        }
    }

    // pub fn get_frame_viewport(&self) -> [Color32; LCD_WIDTH * LCD_HEIGHT] {
    //     let mut viewport = [Color32::WHITE; LCD_WIDTH * LCD_HEIGHT];

    //     for y in 0..LCD_HEIGHT {
    //         for x in 0..LCD_WIDTH {
    //             let wrapping_y = (y + (self.regs.scy as usize)) % 256;
    //             let wrapping_x = (x + (self.regs.scx as usize)) % 256;

    //             let index = wrapping_y * 256 + wrapping_x;
    //             let new_index = y * LCD_WIDTH + x;

    //             let bg_pixel = self.bg_map[index];
    //             viewport[new_index] = bg_pixel;
    //         }
    //     }

    //     viewport
    // }

    fn get_current_line(&self, memory: &mut [u8]) -> Vec<u8> {
        let mut current_line: Vec<u8> = Vec::new();

        // bg enable
        if self.regs.is_bg_enabled() {
            let tile_map_area = if self.regs.lcdc & 0b1000 != 0 {
                0x9C00
            } else {
                0x9800
            };

            let yy = self.regs.ly + self.regs.scy;

            let unsigned_addressing = self.regs.lcdc & 0b10000 != 0;
            let tile_map_start = tile_map_area + (((yy / 8) as usize) * 0x20);

            // println!("{:#06X} + {:#06X} + {:#06X} = {:#06X}", tile_map_area, (((yy / 8) as usize) * 0x20), self.regs.scx, tile_map_start);

            for index in tile_map_start..=(tile_map_start + 0x1F) {
                // println!("{:#06X}, scy: {}", index, self.regs.scy);

                let line_index = (memory[index] as usize) * 16;
                let ly_bytes = (yy % 8) as usize;

                if unsigned_addressing {
                    let first_byte = memory[0x8000 + (line_index) + (2 * ly_bytes)];
                    let second_byte = memory[0x8000 + (line_index) + (2 * ly_bytes + 1)];

                    for i in (0..8).rev() {
                        let lsb = (first_byte & (1 << i)) >> i;
                        let msb = (second_byte & (1 << i)) >> i;

                        current_line.push(msb << 1 | lsb);
                    }
                } else {
                    // println!("SIGNED ADDRESSING");

                    let first_byte = if memory[index] <= 127 {
                        memory[0x9000 + (line_index) + (2 * ly_bytes)]
                    } else {
                        let line_index = ((memory[index] as usize) % 128) * 16;
                        memory[0x8800 + ((line_index) + (2 * ly_bytes))]
                    };

                    let second_byte = if memory[index] <= 127 {
                        memory[0x9000 + (line_index) + (2 * ly_bytes + 1)]
                    } else {
                        let line_index = ((memory[index] as usize) % 128) * 16;
                        memory[0x8800 + ((line_index) + (2 * ly_bytes + 1))]
                    };

                    for i in (0..8).rev() {
                        let lsb = (first_byte & (1 << i)) >> i;
                        let msb = (second_byte & (1 << i)) >> i;

                        current_line.push((msb << 1) | lsb);
                    }
                }
            }

            // if self.regs.is_window_enabled() {
            //     let window_tile_map_area = if self.regs.lcdc & 0b1000000 != 0 {
            //         0x9C00
            //     } else {
            //         0x9800
            //     };

            //     let window_tile_map_start = window_tile_map_area + (((self.regs.ly / 8) as usize) * 0x20);

            //     for index in window_tile_map_start..=(window_tile_map_start + 0x1F) {
            //         if unsigned_addressing {

            //         }
            //     }
            // }

            current_line.rotate_left(self.regs.scx as usize);
            current_line
        } else {
            log::info!("bg is disabled, ly {}", self.regs.ly);
            vec![0b00; LCD_WIDTH]
        }
    }

    fn draw_current_line(&mut self) {
        let y = self.regs.ly as usize;

        // for i in 0..LCD_WIDTH {
        //     self.bg_map[y * 256 + i] = match self.current_line[i] {
        //         0b00 => bgp_color_from_value(self.regs.bgp & 0b11),
        //         0b01 => bgp_color_from_value((self.regs.bgp & 0b1100) >> 2),
        //         0b10 => bgp_color_from_value((self.regs.bgp & 0b110000) >> 4),
        //         0b11 => bgp_color_from_value((self.regs.bgp & 0b11000000) >> 6),
        //         _ => unreachable!(),
        //     }
        // }

        for i in 0..LCD_WIDTH {
            self.test_map[y * LCD_WIDTH + i] = match self.current_line[i] {
                0b00 => bgp_color_from_value(self.regs.bgp & 0b11),
                0b01 => bgp_color_from_value((self.regs.bgp & 0b1100) >> 2),
                0b10 => bgp_color_from_value((self.regs.bgp & 0b110000) >> 4),
                0b11 => bgp_color_from_value((self.regs.bgp & 0b11000000) >> 6),
                _ => unreachable!(),
            }
        }

        self.current_line.clear();
    }

    fn change_mode(&mut self, to: Mode, interrupt_handler: &mut InterruptHandler) {
        if self.current_mode != to {
            match to {
                Mode::VBlank => {
                    self.regs.stat |= to as u8;
                    interrupt_handler.request_interrupt(Interrupt::VBlank);
                }
                Mode::HBlank => self.regs.stat &= !(0b11),
                _ => self.regs.stat |= to as u8,
            }

            self.current_mode = to;
        }
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
