#[derive(Clone, Copy)]
pub struct TileAttribute {
    pub bg_to_oam: BgOamPrio,
    pub v_flip: bool,
    pub h_flip: bool,
    pub vram_bank: u8,
    pub bgp: u8,
}

impl From<u8> for TileAttribute {
    fn from(value: u8) -> Self {
        Self {
            bg_to_oam: BgOamPrio::from((value & 0x80) >> 7),
            v_flip: ((value & (0x40)) >> 6) != 0,
            h_flip: ((value & (0x20)) >> 5) != 0,
            vram_bank: (value & 0x08) >> 3,
            bgp: value & 0x07,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BgOamPrio {
    OAMPrio,
    BGPrio,
}

impl From<u8> for BgOamPrio {
    fn from(value: u8) -> Self {
        match value {
            0 => BgOamPrio::OAMPrio,
            1 => BgOamPrio::BGPrio,
            _ => unreachable!(),
        }
    }
}
