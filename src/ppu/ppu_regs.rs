#[derive(Default)]
pub(crate) struct PPURegisters {
    pub lcdc: u8,
    pub stat: u8,
    pub scy: u8,
    pub scx: u8,
    pub ly: u8,
    pub lyc: u8,
    pub wy: u8,
    pub wx: u8,
    pub bgp: u8,
    pub opb0: u8,
    pub opb1: u8,
    pub dma: u8,
}

impl PPURegisters {
    pub fn is_lcd_on(&self) -> bool {
        self.lcdc & (1 << 7) != 0
    }

    pub fn ly_lyc(&self) -> bool {
        self.ly == self.lyc
    }

    pub fn is_bg_enabled(&self) -> bool {
        self.lcdc & 0x1 != 0
    }

    pub fn is_obj_enabled(&self) -> bool {
        self.lcdc & 0x2 != 0
    }

    pub fn is_window_enabled(&self) -> bool {
        self.lcdc & 0x20 != 0
    }

    pub fn is_window_visible(&self) -> bool {
        (0..=166).contains(&self.wx) && (0..=143).contains(&self.wy)
    }

    pub fn is_sprite_8x8(&self) -> bool {
        self.lcdc & 0x4 == 0
    }
}
