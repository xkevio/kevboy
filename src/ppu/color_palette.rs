use eframe::epaint::Color32;

pub(super) const LCD_WHITE: Color32 = Color32::from_rgb(127, 134, 15);
pub(super) const LCD_LIGHT_GRAY: Color32 = Color32::from_rgb(87, 124, 68);
pub(super) const LCD_GRAY: Color32 = Color32::from_rgb(54, 93, 72);
pub(super) const LCD_BLACK: Color32 = Color32::from_rgb(42, 69, 59);

// const LCD_TRANSPARENCY: Color32 = Color32::fr;

#[derive(Clone, Copy)]
pub enum Palette {
    BGP(u8),
    OBP0(u8),
    OBP1(u8),
}

pub(super) fn convert_to_color(value: u8, palette: Palette) -> Color32 {
    match palette {
        Palette::BGP(bgp) => match value {
            0b00 => bgp_color_from_value(bgp & 0b11),
            0b01 => bgp_color_from_value((bgp & 0b1100) >> 2),
            0b10 => bgp_color_from_value((bgp & 0b110000) >> 4),
            0b11 => bgp_color_from_value((bgp & 0b11000000) >> 6),
            _ => unreachable!(),
        },
        Palette::OBP0(obp0) => match value {
            // 0b00 => ,
            0b01 => obp_color_from_value((obp0 & 0b1100) >> 2),
            0b10 => obp_color_from_value((obp0 & 0b110000) >> 4),
            0b11 => obp_color_from_value((obp0 & 0b11000000) >> 6),
            _ => unreachable!(),
        },
        Palette::OBP1(obp1) => match value {
            // 0b00 => LCD_TRANSPARENCY,
            0b01 => obp_color_from_value((obp1 & 0b1100) >> 2),
            0b10 => obp_color_from_value((obp1 & 0b110000) >> 4),
            0b11 => obp_color_from_value((obp1 & 0b11000000) >> 6),
            _ => unreachable!(),
        },
    }
}

fn bgp_color_from_value(value: u8) -> Color32 {
    match value {
        0b00 => LCD_WHITE,
        0b01 => LCD_LIGHT_GRAY,
        0b10 => LCD_GRAY,
        0b11 => LCD_BLACK,
        _ => unreachable!(),
    }
}

fn obp_color_from_value(value: u8) -> Color32 {
    match value {
        0b00 => LCD_WHITE,
        0b01 => LCD_LIGHT_GRAY,
        0b10 => LCD_GRAY,
        0b11 => LCD_BLACK,
        _ => unreachable!(),
    }
}
