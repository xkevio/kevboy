#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    pub y_pos: u8,
    pub x_pos: u8,
    pub tile_index: u8,
    attr: u8,
}

/// OAM goes from: $FE00-$FE9F.
/// We have 10 sprites per scanline and sort by x-coordinates
/// and OAM order in DMG or just order in CGB.
pub fn get_current_sprites_per_line(
    oam: &[u8],
    ly: u8,
    height_mode: bool,
    cgb: bool,
) -> Vec<Sprite> {
    let mut sprites: Vec<Sprite> = Vec::new();

    for attributes in oam.chunks(4) {
        let y = attributes[0]; // y + 16, bottom line of sprite
        let sprite_height = if height_mode { 8 } else { 16 };

        let ly_range = (y as i16 - 16)..(y as i16 - 16 + sprite_height as i16);

        if ly_range.contains(&(ly as i16)) && sprites.len() < 10 {
            let x = attributes[1]; // x + 8, right side of sprite
            let upper_tile_index = attributes[2];
            let attr = attributes[3];

            sprites.push(Sprite::new(y, x, upper_tile_index, attr));
        }
    }

    if !cgb {
        sprites.sort_by(|a, b| a.x_pos.cmp(&b.x_pos));
    }
    sprites
}

impl Sprite {
    pub fn new(y_pos: u8, x_pos: u8, tile_index: u8, attr: u8) -> Self {
        Self {
            y_pos,
            x_pos,
            tile_index,
            attr,
        }
    }

    pub fn is_y_flipped(&self) -> bool {
        self.attr & 0x40 != 0
    }

    pub fn is_x_flipped(&self) -> bool {
        self.attr & 0x20 != 0
    }

    /// 0 = OBP0
    ///
    /// 1 = OBP1
    pub fn get_dmg_obp_num(&self) -> u8 {
        (self.attr & 0x10) >> 4
    }

    pub fn get_cgb_obp_num(&self) -> u8 {
        self.attr & 0x07
    }

    pub fn vbk(&self) -> u8 {
        (self.attr & 0x08) >> 3
    }

    /// True if bit 7 of byte 3 is 0
    pub fn is_obj_prio(&self) -> bool {
        self.attr & 0x80 == 0
    }
}
