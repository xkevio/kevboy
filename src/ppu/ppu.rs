use eframe::epaint::Color32;

use crate::{
    cpu::interrupts::{Interrupt, InterruptHandler},
    mmu::mmio::MMIO,
    ppu::{color_palette::*, ppu_regs::PPURegisters, sprite, sprite::Sprite},
};

pub const LCD_WIDTH: usize = 160;
pub const LCD_HEIGHT: usize = 144;

#[derive(PartialEq, Clone, Copy, Debug)]
enum Mode {
    HBlank = 0,
    VBlank = 0b1,
    Mode2 = 0b10,
    Mode3 = 0b11,
}

#[allow(clippy::upper_case_acronyms)]
pub struct PPU {
    // LCD screen array, current viewport
    pub frame_buffer: [Color32; LCD_WIDTH * LCD_HEIGHT],
    current_line: Vec<Color32>,

    regs: PPURegisters,
    cycles_passed: i16,

    current_mode: Mode,
    stat_block: bool,
    dma_pending: bool,

    internal_window_line: u8,
    current_sprites: Vec<Sprite>,
}

impl MMIO for PPU {
    fn read(&mut self, address: u16) -> u8 {
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
            0xFF46 => {
                self.regs.dma = value;
                self.dma_pending = true;
            }
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
    pub fn new() -> Self {
        Self {
            frame_buffer: [LCD_WHITE; LCD_WIDTH * LCD_HEIGHT],
            current_line: Vec::new(),

            regs: PPURegisters::default(),
            cycles_passed: 0,

            current_mode: Mode::HBlank,
            stat_block: false,
            dma_pending: false,

            internal_window_line: 0,
            current_sprites: Vec::new(),
        }
    }

    // should tick 4 times per m-cycle
    // 456 clocks per scanline
    // 80 (Mode2) - 172 (Mode3) - 204 (HBlank) - VBlank
    pub fn tick(&mut self, vram: &[u8], oam: &[u8], interrupt_handler: &mut InterruptHandler) {
        if self.regs.is_lcd_on() {
            if self.regs.ly_lyc() {
                self.regs.stat |= 0b100;
            } else {
                self.regs.stat &= !(0b100);
            }

            // ly=lyc check happens on the 4th cycle of the line
            if self.cycles_passed == 4 && self.check_stat_interrupt() {
                interrupt_handler.request_interrupt(Interrupt::STAT);
            }

            if self.regs.ly < 144 {
                self.handle_line0_to_line143(vram, oam, interrupt_handler);
            } else {
                if self.cycles_passed >= 456 {
                    self.regs.ly += 1;

                    if self.regs.ly > 153 {
                        self.regs.ly = 0;
                    }

                    self.cycles_passed = -1;
                }
            }

            self.cycles_passed += 1;
        }
    }

    // --------------------------
    //          DMA
    // --------------------------

    pub fn is_dma_pending(&self) -> bool {
        self.dma_pending
    }

    pub fn reset_dma(&mut self) {
        self.dma_pending = false
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
        vram: &[u8],
        oam: &[u8],
        interrupt_handler: &mut InterruptHandler,
    ) {
        match self.cycles_passed {
            0 => {
                self.change_mode(Mode::Mode2, Some(interrupt_handler));

                // scan oam for sprites
                self.current_sprites = sprite::get_current_sprites_per_line(
                    self.regs.ly,
                    self.regs.is_sprite_8x8(),
                    oam,
                );
            }
            80 => {
                self.change_mode(Mode::Mode3, Some(interrupt_handler));

                // "draw" pixels into current line
                if self.current_line.is_empty() {
                    self.current_line = self.get_current_line(vram);
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
                    self.internal_window_line = 0;
                } else {
                    self.change_mode(Mode::Mode2, Some(interrupt_handler));
                }

                self.cycles_passed = -1;
            }
            _ => {}
        }
    }

    fn get_current_line(&mut self, vram: &[u8]) -> Vec<Color32> {
        let mut current_line: Vec<Color32> = Vec::new();

        // bg enable
        if self.regs.is_bg_enabled() {
            let unsigned_addressing = self.regs.lcdc & 0b10000 != 0;

            let bg_tile_map_area = if self.regs.lcdc & 0b1000 == 0 {
                0x9800 - 0x8000
            } else {
                0x9C00 - 0x8000
            };

            let adjusted_y = self.regs.ly + self.regs.scy;
            let tile_map_start = bg_tile_map_area + (((adjusted_y / 8) as usize) * 0x20);

            for index in tile_map_start..=(tile_map_start + 0x1F) {
                let tile_row = self.get_tile_row(vram, unsigned_addressing, index, adjusted_y);
                current_line.extend(tile_row);
            }

            // Apply SCX to the background layer
            current_line.rotate_left(self.regs.scx as usize);

            // Draw window over bg if enabled and visible
            if self.regs.is_window_enabled()
                && self.regs.is_window_visible()
                && self.regs.ly >= self.regs.wy
            {
                let win_tile_map_area = if self.regs.lcdc & 0x40 == 0 {
                    0x9800 - 0x8000
                } else {
                    0x9C00 - 0x8000
                };

                let win_y = self.internal_window_line;
                let tile_map_start = win_tile_map_area + (((win_y / 8) as usize) * 0x20);

                for (j, index) in (tile_map_start..=(tile_map_start + 0x1F)).enumerate() {
                    let tile_row = self.get_tile_row(vram, unsigned_addressing, index, win_y);

                    for i in 0..8 {
                        if (((self.regs.wx - 7) as usize) + i + (j * 8)) < 256 {
                            current_line[((self.regs.wx - 7) as usize) + i + (j * 8)] = tile_row[i];
                        }
                    }
                }
            }
        }

        // Fill current_line when empty aka when bg / window were disabled
        if current_line.is_empty() {
            current_line = vec![LCD_WHITE; LCD_WIDTH];
        }

        // ----------------------------
        //      Sprites
        // ----------------------------

        if self.regs.is_obj_enabled() {
            for sprite in self.current_sprites.iter().rev() {
                let upper_tile = sprite.tile_index & 0xFE;
                let lower_tile = sprite.tile_index | 0x1;

                let current_tile = if self.regs.is_sprite_8x8() {
                    sprite.tile_index
                } else {
                    if sprite.is_y_flipped() {
                        if (self.regs.ly + sprite.y_pos) % 16 >= 8 {
                            upper_tile
                        } else {
                            lower_tile
                        }
                    } else {
                        if (self.regs.ly + sprite.y_pos) % 16 >= 8 {
                            lower_tile
                        } else {
                            upper_tile
                        }
                    }
                };

                let sprite_tile = (current_tile as usize) * 16;
                let ly_bytes = ((self.regs.ly - sprite.y_pos) % 8) as usize;

                let palette = if sprite.get_obp_num() == 0 {
                    Palette::OBP(self.regs.opb0)
                } else {
                    Palette::OBP(self.regs.opb1)
                };

                let first_byte = if !sprite.is_y_flipped() {
                    vram[sprite_tile + (2 * ly_bytes)]
                } else {
                    vram[sprite_tile + (2 * (7 - ly_bytes))]
                };

                let second_byte = if !sprite.is_y_flipped() {
                    vram[sprite_tile + (2 * ly_bytes + 1)]
                } else {
                    vram[sprite_tile + (2 * (7 - ly_bytes) + 1)]
                };

                for i in (0..8).rev() {
                    let lsb = (first_byte & (1 << i)) >> i;
                    let msb = (second_byte & (1 << i)) >> i;

                    let x_flip = if sprite.is_x_flipped() { i } else { 7 - i };
                    let x = (sprite.x_pos + x_flip) as usize;

                    if msb << 1 | lsb != 0 {
                        let color = if sprite.is_obj_prio() {
                            convert_to_color(msb << 1 | lsb, palette)
                        } else {
                            if current_line[x] == convert_to_color(0, Palette::BGP(self.regs.bgp)) {
                                convert_to_color(msb << 1 | lsb, palette)
                            } else {
                                current_line[x]
                            }
                        };

                        current_line[x] = color;
                    }
                }
            }
        }

        current_line
    }

    fn get_tile_row(
        &self,
        vram: &[u8],
        unsigned_addressing: bool,
        index: usize,
        y: u8,
    ) -> [Color32; 8] {
        let mut current_line: [Color32; 8] = [LCD_WHITE; 8];

        let line_index = (vram[index] as usize) * 16;
        let ly_bytes = (y % 8) as usize;

        if unsigned_addressing {
            let first_byte = vram[line_index + (2 * ly_bytes)];
            let second_byte = vram[line_index + (2 * ly_bytes + 1)];

            for i in (0..8).rev() {
                let lsb = (first_byte & (1 << i)) >> i;
                let msb = (second_byte & (1 << i)) >> i;

                current_line[7 - i] = convert_to_color(msb << 1 | lsb, Palette::BGP(self.regs.bgp));
            }
        } else {
            let first_byte = if vram[index] <= 127 {
                vram[(0x9000 - 0x8000) + line_index + (2 * ly_bytes)]
            } else {
                let line_index = ((vram[index] as usize) % 128) * 16;
                vram[(0x8800 - 0x8000) + line_index + (2 * ly_bytes)]
            };

            let second_byte = if vram[index] <= 127 {
                vram[(0x9000 - 0x8000) + line_index + (2 * ly_bytes + 1)]
            } else {
                let line_index = ((vram[index] as usize) % 128) * 16;
                vram[(0x8800 - 0x8000) + line_index + (2 * ly_bytes + 1)]
            };

            for i in (0..8).rev() {
                let lsb = (first_byte & (1 << i)) >> i;
                let msb = (second_byte & (1 << i)) >> i;

                current_line[7 - i] = convert_to_color(msb << 1 | lsb, Palette::BGP(self.regs.bgp));
            }
        }

        current_line
    }

    fn draw_current_line(&mut self) {
        let y = self.regs.ly as usize;

        for i in 0..LCD_WIDTH {
            self.frame_buffer[y * LCD_WIDTH + i] = self.current_line[i];
        }

        self.current_line.clear();
        self.current_sprites.clear();
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
