use crate::{
    cpu::interrupts::{Interrupt, InterruptHandler},
    mmu::mmio::MMIO,
    ppu::{color_palette::*, ppu_regs::PPURegisters, sprite, sprite::Sprite},
};

use super::tile_attributes::TileAttribute;

// --------- PPU constants ---------
pub const LCD_WIDTH: usize = 160;
pub const LCD_HEIGHT: usize = 144;

const MODE3_START: i16 = 80;
const HBLANK_START: i16 = 252;
const LINE_END: i16 = 455;
// --------------------------------

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Mode {
    HBlank = 0,
    VBlank = 0b1,
    Mode2 = 0b10,
    Mode3 = 0b11,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum DMATransferState {
    Pending,
    Transferring,
    Disabled,
}

#[allow(clippy::upper_case_acronyms)]
pub struct PPU {
    /// LCD screen array, current viewport (double buffering)
    pub frame_buffer: Box<[ScreenColor; LCD_WIDTH * LCD_HEIGHT]>,
    pub ui_frame_buffer: Box<[ScreenColor; LCD_WIDTH * LCD_HEIGHT]>,

    /// Raw 256x256 background for debugging purposes
    pub raw_frame: Vec<ScreenColor>,

    /// Color RAM for CGB mode, stored as RGB555
    bg_cram: [u8; 64],
    bgpi: u8,

    obj_cram: [u8; 64],
    obpi: u8,

    /// Contains pixels for the current line
    current_line: Vec<ScreenColor>,
    /// Contains up to 10 sprites that will be rendered this line
    current_sprites: Vec<Sprite>,

    /// All PPU registers needed for DMG, MMIO
    regs: PPURegisters,
    /// Current dot the PPU is at relative to beginning of a line
    dots: i16,

    /// Current mode, based on bits 1-0 of STAT
    current_mode: Mode,
    /// Stores previous STAT line to detect rising edge
    stat_block: bool,
    /// Current DMA state to properly delay DMA transfer one m-cycle
    dma_state: DMATransferState,

    internal_window_line: u8,
    cgb: bool,
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
            0xFF68 => self.bgpi,
            0xFF69 => {
                let address = self.bgpi & 0x3F;
                self.bg_cram[address as usize]
            }
            0xFF6A => self.obpi,
            0xFF6B => {
                let address = self.obpi & 0x3F;
                self.obj_cram[address as usize]
            }
            _ => unreachable!(),
        }
    }

    fn write(&mut self, _address: u16, _value: u8) {
        unimplemented!()
    }

    fn write_with_callback(&mut self, address: u16, value: u8, cb: &mut InterruptHandler) {
        match address {
            0xFF40 => {
                self.regs.lcdc = value;

                if value & 0x80 == 0 {
                    self.turn_lcd_off();
                } else {
                    // LY=LYC comparison clock starts again after LCD is enabled,
                    // this passes the stat_lyc_on_off test.
                    if self.regs.ly_lyc() {
                        self.regs.stat |= 0b100;
                    } else {
                        self.regs.stat &= !(0b100);
                    }

                    if self.check_stat_interrupt() {
                        cb.request_interrupt(Interrupt::STAT);
                    }
                }
            }
            0xFF41 => {
                self.regs.stat = (1 << 7) | (value & !(0b111)) | (self.regs.stat & 0b111);

                if self.check_stat_interrupt() {
                    cb.request_interrupt(Interrupt::STAT);
                }
            }
            0xFF42 => self.regs.scy = value,
            0xFF43 => self.regs.scx = value,
            0xFF44 => {} // LY should be read-only from Bus
            0xFF45 => {
                self.regs.lyc = value;

                if self.regs.is_lcd_on() {
                    if self.regs.ly_lyc() {
                        self.regs.stat |= 0b100;
                    } else {
                        self.regs.stat &= !(0b100);
                    }
                }

                if self.check_stat_interrupt() {
                    cb.request_interrupt(Interrupt::STAT);
                }
            }
            0xFF46 => {
                self.regs.dma = value;
                self.dma_state = DMATransferState::Pending;
            }
            0xFF47 => self.regs.bgp = value,
            0xFF48 => self.regs.opb0 = value,
            0xFF49 => self.regs.opb1 = value,
            0xFF4A => self.regs.wy = value,
            0xFF4B => self.regs.wx = value,
            0xFF68 => self.bgpi = value,
            0xFF69 => {
                let auto_inc = (self.bgpi & 0x80) >> 7 != 0;
                let address = self.bgpi & 0x3F;

                if auto_inc {
                    self.bgpi += 1;
                }
                self.bg_cram[address as usize] = value;
            }
            0xFF6A => self.obpi = value,
            0xFF6B => {
                let auto_inc = (self.obpi & 0x80) >> 7 != 0;
                let address = self.obpi & 0x3F;

                if auto_inc {
                    self.obpi += 1;
                }
                self.obj_cram[address as usize] = value;
            }
            _ => unreachable!(),
        }
    }
}

impl PPU {
    pub fn new() -> Self {
        Self {
            frame_buffer: vec![ScreenColor::White; LCD_WIDTH * LCD_HEIGHT]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            ui_frame_buffer: vec![ScreenColor::White; LCD_WIDTH * LCD_HEIGHT]
                .into_boxed_slice()
                .try_into()
                .unwrap(),

            raw_frame: vec![ScreenColor::White; 256 * 256],
            bg_cram: [0xFF; 64],
            bgpi: 0xC8,

            obj_cram: [0xFF; 64],
            obpi: 0xD0,

            current_line: Vec::new(),
            current_sprites: Vec::new(),

            regs: PPURegisters::default(),
            dots: 0,

            current_mode: Mode::HBlank,
            stat_block: false,
            dma_state: DMATransferState::Disabled,

            internal_window_line: 0,
            cgb: false,
        }
    }

    // Should tick 4 times per m-cycle
    // 456 clocks per scanline (see lengths below)
    // 80 (Mode2) - 172 (Mode3) - 204 (HBlank) - VBlank
    pub fn tick(
        &mut self,
        vram: &[[u8; 0x2000]],
        oam: &[u8],
        interrupt_handler: &mut InterruptHandler,
    ) {
        if self.regs.is_lcd_on() {
            // LY = 0 after lcd turn on: special behavior
            match &self.current_mode {
                Mode::Mode2 => {
                    if self.dots >= MODE3_START {
                        self.change_mode(Mode::Mode3, interrupt_handler);
                    }

                    // Scan OAM for (up to) 10 sprites
                    if self.dots == 0 {
                        self.current_sprites = sprite::get_current_sprites_per_line(
                            oam,
                            self.regs.ly,
                            self.regs.is_sprite_8x8(),
                            self.cgb,
                        );
                    }
                }
                Mode::Mode3 => {
                    // Get pixels of current line ("draw" at end of mode)
                    if self.dots >= HBLANK_START {
                        // Important: draw later during Mode3 to fix parts of pocket.gb
                        if self.current_line.is_empty() {
                            self.current_line = self.get_current_line(vram);
                        }

                        if self.regs.is_window_visible()
                            && self.regs.ly >= self.regs.wy
                            && self.regs.is_window_enabled()
                        {
                            self.internal_window_line += 1;
                        }

                        self.draw_current_line(); // -> side effect: clears self.current_line and self.current_sprites
                        self.change_mode(Mode::HBlank, interrupt_handler);
                    }
                }
                Mode::HBlank => {
                    if self.dots >= LINE_END {
                        self.regs.ly += 1;

                        // Check STAT irq for LY change
                        if self.regs.ly_lyc() {
                            self.regs.stat |= 0b100;
                        } else {
                            self.regs.stat &= !(0b100);
                        }

                        if self.check_stat_interrupt() {
                            interrupt_handler.request_interrupt(Interrupt::STAT);
                        }

                        if self.regs.ly >= 144 {
                            self.change_mode(Mode::VBlank, interrupt_handler);
                            self.internal_window_line = 0;

                            // Swap buffers to avoid screen tearing on VBlank
                            std::mem::swap(&mut self.frame_buffer, &mut self.ui_frame_buffer);
                        } else {
                            self.change_mode(Mode::Mode2, interrupt_handler);
                        }

                        self.dots = -1;
                    }
                }
                Mode::VBlank => {
                    if self.dots >= LINE_END {
                        self.regs.ly += 1;

                        if self.regs.ly_lyc() {
                            self.regs.stat |= 0b100;
                        } else {
                            self.regs.stat &= !(0b100);
                        }

                        if self.check_stat_interrupt() {
                            interrupt_handler.request_interrupt(Interrupt::STAT);
                        }

                        // TODO: 1 m-cycle after ly is set to 153, it is set to 0 NOT immediately
                        if self.regs.ly >= 153 {
                            self.regs.ly = 0;

                            if self.regs.ly_lyc() {
                                self.regs.stat |= 0b100;
                            } else {
                                self.regs.stat &= !(0b100);
                            }

                            self.change_mode(Mode::Mode2, interrupt_handler);
                            self.dump_bg_map(vram);
                        }

                        self.dots = -1;
                    }
                }
            };

            self.dots += 1;
        }
    }

    pub fn enable_cgb(&mut self) {
        self.cgb = true;
    }

    // --------------------------
    //          DMA
    // --------------------------

    pub fn get_dma_state(&self) -> DMATransferState {
        self.dma_state
    }

    pub fn set_dma_enable(&mut self) {
        self.dma_state = DMATransferState::Transferring;
    }

    pub fn reset_dma(&mut self) {
        self.dma_state = DMATransferState::Disabled;
    }

    // --------------------------

    fn turn_lcd_off(&mut self) {
        self.regs.ly = 0;
        self.internal_window_line = 0;

        self.dots = 0;
        self.regs.stat &= !(0b11);
        self.current_mode = Mode::HBlank;
    }

    // -------------------------
    // Rendering logic (bg & win)
    // -------------------------

    // -------- DEBUGGING STUFF --------

    /// Dumps 256x256 BG map for the vram viewer
    fn dump_bg_map(&mut self, vram: &[[u8; 0x2000]]) {
        let mut current_line: Vec<ScreenColor> = Vec::with_capacity(256);

        for i in 0..=255 {
            let unsigned_addressing = self.regs.lcdc & 0b10000 != 0;

            let bg_tile_map_area = if self.regs.lcdc & 0b1000 == 0 {
                0x9800 - 0x8000
            } else {
                0x9C00 - 0x8000
            };

            let adjusted_y = i;
            let tile_map_start = bg_tile_map_area + (((adjusted_y / 8) as usize) * 0x20);

            for index in tile_map_start..=(tile_map_start + 0x1F) {
                let tile_row = self.get_tile_row(vram, unsigned_addressing, index, adjusted_y);
                current_line.extend(tile_row);
            }

            for x in 0..256 {
                self.raw_frame[i as usize * 256 + x] = current_line[x];
            }
            current_line.clear();
        }
    }

    // -------- ACTUAL RENDERING --------

    fn get_current_line(&self, vram: &[[u8; 0x2000]]) -> Vec<ScreenColor> {
        let bg_win_line = self.get_bg_win_line(vram);
        let sprite_line = self.get_sprite_line(vram, &bg_win_line);

        sprite_line
    }

    fn get_bg_win_line(&self, vram: &[[u8; 0x2000]]) -> Vec<ScreenColor> {
        let mut current_line: Vec<ScreenColor> = Vec::with_capacity(256);

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

            // Apply SCX to the current scanline in the background layer
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
                        let signed_wx = self.regs.wx as i16;
                        let win_index = (signed_wx - 7) + i as i16 + (j as i16 * 8);
                        if (0..256).contains(&win_index) {
                            current_line[win_index as usize] = tile_row[i];
                        }
                    }
                }
            }
        }

        // Fill current_line when empty aka when bg / window were disabled
        if current_line.is_empty() {
            current_line = vec![ScreenColor::White; 256];
        }

        current_line
    }

    // -------------------------
    // Sprites
    // -------------------------

    fn get_sprite_line(
        &self,
        vram: &[[u8; 0x2000]],
        current_line: &[ScreenColor],
    ) -> Vec<ScreenColor> {
        let mut current_line: Vec<ScreenColor> = Vec::from(current_line);

        if self.regs.is_obj_enabled() {
            for sprite in self.current_sprites.iter().rev() {
                let vbk = sprite.vbk() as usize;
                let upper_tile = sprite.tile_index & 0xFE;
                let lower_tile = sprite.tile_index | 0x1;

                let real_x_pos = sprite.x_pos as i16 - 8;
                let real_y_pos = sprite.y_pos as i16 - 16;

                let current_tile = if self.regs.is_sprite_8x8() {
                    sprite.tile_index
                } else {
                    if sprite.is_y_flipped() {
                        if real_y_pos.abs_diff(self.regs.ly as i16) >= 8 {
                            upper_tile
                        } else {
                            lower_tile
                        }
                    } else {
                        if real_y_pos.abs_diff(self.regs.ly as i16) >= 8 {
                            lower_tile
                        } else {
                            upper_tile
                        }
                    }
                };

                let sprite_tile = (current_tile as usize) * 16;
                let ly_bytes = (real_y_pos.abs_diff(self.regs.ly as i16) % 8) as usize;

                let palette = if !self.cgb {
                    if sprite.get_dmg_obp_num() == 0 {
                        Palette::OBP(self.regs.opb0)
                    } else {
                        Palette::OBP(self.regs.opb1)
                    }
                } else {
                    Palette::OBP(sprite.get_cgb_obp_num())
                };

                let first_byte = if !sprite.is_y_flipped() {
                    vram[vbk][sprite_tile + (2 * ly_bytes)]
                } else {
                    vram[vbk][sprite_tile + (2 * (7 - ly_bytes))]
                };

                let second_byte = if !sprite.is_y_flipped() {
                    vram[vbk][sprite_tile + (2 * ly_bytes + 1)]
                } else {
                    vram[vbk][sprite_tile + (2 * (7 - ly_bytes) + 1)]
                };

                for i in (0..8).rev() {
                    let lsb = (first_byte & (1 << i)) >> i;
                    let msb = (second_byte & (1 << i)) >> i;

                    let x_flip = if sprite.is_x_flipped() { i } else { 7 - i };

                    if real_x_pos + x_flip < 0 {
                        continue;
                    }

                    let x = (real_x_pos + x_flip) as usize;
                    if (msb << 1 | lsb) != 0 {
                        let color = if sprite.is_obj_prio() {
                            convert_to_color(msb << 1 | lsb, palette, self.cgb, &self.obj_cram)
                        } else {
                            if current_line[x]
                                == convert_to_color(
                                    0,
                                    palette, // bug fix?
                                    self.cgb,
                                    &self.obj_cram,
                                )
                            {
                                convert_to_color(msb << 1 | lsb, palette, self.cgb, &self.obj_cram)
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

    /// Gets the 8 pixels of the current bg/win tile
    ///
    /// Can't use it for sprites because of the obj prio bit and flip bits
    fn get_tile_row(
        &self,
        vram: &[[u8; 0x2000]],
        unsigned_addressing: bool,
        index: usize,
        y: u8,
    ) -> [ScreenColor; 8] {
        let mut current_line: [ScreenColor; 8] = [ScreenColor::White; 8];

        let tile_attribute = TileAttribute::from(vram[1][index]);
        let vbk = if self.cgb {
            tile_attribute.vram_bank as usize
        } else {
            0
        };
        let bgp = if self.cgb {
            tile_attribute.bgp
        } else {
            self.regs.bgp
        };

        let line_index = (vram[vbk][index] as usize) * 16;
        let ly_bytes = (y % 8) as usize;

        if unsigned_addressing {
            let first_byte = vram[vbk][line_index + (2 * ly_bytes)];
            let second_byte = vram[vbk][line_index + (2 * ly_bytes + 1)];

            for i in (0..8).rev() {
                let lsb = (first_byte & (1 << i)) >> i;
                let msb = (second_byte & (1 << i)) >> i;
                let h_index = if tile_attribute.h_flip && self.cgb {
                    i
                } else {
                    7 - i
                };

                current_line[h_index] =
                    convert_to_color(msb << 1 | lsb, Palette::BGP(bgp), self.cgb, &self.bg_cram);
            }
        } else {
            let first_byte = if vram[vbk][index] <= 127 {
                vram[vbk][(0x9000 - 0x8000) + line_index + (2 * ly_bytes)]
            } else {
                let line_index = ((vram[vbk][index] as usize) % 128) * 16;
                vram[vbk][(0x8800 - 0x8000) + line_index + (2 * ly_bytes)]
            };

            let second_byte = if vram[vbk][index] <= 127 {
                vram[vbk][(0x9000 - 0x8000) + line_index + (2 * ly_bytes + 1)]
            } else {
                let line_index = ((vram[vbk][index] as usize) % 128) * 16;
                vram[vbk][(0x8800 - 0x8000) + line_index + (2 * ly_bytes + 1)]
            };

            for i in (0..8).rev() {
                let lsb = (first_byte & (1 << i)) >> i;
                let msb = (second_byte & (1 << i)) >> i;
                let h_index = if tile_attribute.h_flip && self.cgb {
                    i
                } else {
                    7 - i
                };

                current_line[h_index] =
                    convert_to_color(msb << 1 | lsb, Palette::BGP(bgp), self.cgb, &self.bg_cram);
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

    // ----------------------------
    // PPU STAT irq and mode change
    // ----------------------------

    fn change_mode(&mut self, to: Mode, interrupt_handler: &mut InterruptHandler) {
        match to {
            Mode::VBlank => interrupt_handler.request_interrupt(Interrupt::VBlank),
            _ if to == self.current_mode => return,
            _ => {}
        };

        self.regs.stat &= !(0b11);
        self.regs.stat |= to as u8;
        self.current_mode = to;

        if self.check_stat_interrupt() {
            interrupt_handler.request_interrupt(Interrupt::STAT);
        }
    }

    fn check_stat_interrupt(&mut self) -> bool {
        let prev_stat = self.stat_block;
        let current_stat = (self.regs.ly_lyc() && self.regs.stat & (1 << 6) != 0)
            || ((self.current_mode == Mode::HBlank) && self.regs.stat & (1 << 3) != 0)
            || ((self.current_mode == Mode::Mode2) && self.regs.stat & (1 << 5) != 0)
            || ((self.current_mode == Mode::VBlank)
                && ((self.regs.stat & (1 << 4) != 0) | (self.regs.stat & (1 << 5) != 0)));

        self.stat_block = current_stat;
        !prev_stat && current_stat
    }
}
