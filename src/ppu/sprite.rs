pub struct Sprite {
    y_pos: u8,
    x_pos: u8,
    tile_index: u8,
    attr: u8,
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
    pub fn get_obp_num(&self) -> u8 {
        (self.attr & 0x10) >> 4
    }

    /// True if bit 7 of byte 3 is 0
    pub fn is_obj_prio(&self) -> bool {
        self.attr & 0x80 == 0
    }
}
