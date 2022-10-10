#[derive(Default)]
pub(crate) struct PPURegisters {
    lcdc: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    pub ly: u8,
    lyc: u8,
    wy: u8,
    wx: u8,
    bgp: u8,
    opb0: u8,
    opb1: u8,
}

impl PPURegisters {
    pub fn is_lcd_on(&self) -> bool {
        self.lcdc & (1 << 7) != 0
    }

    pub fn ly_lyc(&self) -> bool {
        self.ly == self.lyc
    }
}

