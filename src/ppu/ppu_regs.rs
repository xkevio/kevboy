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
}

impl PPURegisters {
    pub fn is_lcd_on(&self) -> bool {
        self.lcdc & (1 << 7) != 0
    }

    pub fn ly_lyc(&self) -> bool {
        self.ly == self.lyc
    }
}
