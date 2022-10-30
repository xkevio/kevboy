use eframe::epaint::Color32;

pub(super) const LCD_WHITE: Color32 = Color32::from_rgb(127, 134, 15);
pub(super) const LCD_LIGHT_GRAY: Color32 = Color32::from_rgb(87, 124, 68);
pub(super) const LCD_GRAY: Color32 = Color32::from_rgb(54, 93, 72);
pub(super) const LCD_BLACK: Color32 = Color32::from_rgb(42, 69, 59);

/// Palette enum with the according palette register as the associated value.
/// 
/// `OBP` shall be used for both obp0 and obp1 as long as the correct palette is passed.
#[derive(Clone, Copy)]
pub enum Palette {
    BGP(u8),
    OBP(u8),
}

pub(super) fn convert_to_color(value: u8, palette: Palette) -> Color32 {
    match palette {
        Palette::BGP(bgp) => match value {
            0b00 => color_from_value(bgp & 0b11),
            0b01 => color_from_value((bgp & 0b1100) >> 2),
            0b10 => color_from_value((bgp & 0b110000) >> 4),
            0b11 => color_from_value((bgp & 0b11000000) >> 6),
            _ => unreachable!(),
        },
        Palette::OBP(obp) => match value {
            0b01 => color_from_value((obp & 0b1100) >> 2),
            0b10 => color_from_value((obp & 0b110000) >> 4),
            0b11 => color_from_value((obp & 0b11000000) >> 6),
            _ => unreachable!(),
        },
    }
}

fn color_from_value(value: u8) -> Color32 {
    match value {
        0b00 => LCD_WHITE,
        0b01 => LCD_LIGHT_GRAY,
        0b10 => LCD_GRAY,
        0b11 => LCD_BLACK,
        _ => unreachable!(),
    }
}
