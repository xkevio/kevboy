#[derive(Clone, Copy)]
pub struct TileAttribute {
    bg_to_oam: BgOamPrio,
    v_flip: bool,
    h_flip: bool,
    vram_bank: u8,
    bgp: u8,
}

impl From<u8> for TileAttribute {
    fn from(value: u8) -> Self {
        Self {
            bg_to_oam: BgOamPrio::from((value & 0x80) >> 6),
            v_flip: ((value & (0x40)) >> 5) != 0,
            h_flip: ((value & (0x20)) >> 4) != 0,
            vram_bank: (value & 0x08) >> 2,
            bgp: value & 0x07,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum BgOamPrio {
    OAMPrio,
    BGPrio,
}

impl From<u8> for BgOamPrio {
    fn from(value: u8) -> Self {
        match value {
            0 => BgOamPrio::OAMPrio,
            1 => BgOamPrio::BGPrio,
            _ => unreachable!()
        }
    }
}