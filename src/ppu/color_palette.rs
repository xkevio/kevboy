use eframe::epaint::Color32;

pub(super) const LCD_WHITE: Color32 = Color32::from_rgb(127, 134, 15);
pub(super) const LCD_LIGHT_GRAY: Color32 = Color32::from_rgb(87, 124, 68);
pub(super) const LCD_GRAY: Color32 = Color32::from_rgb(54, 93, 72);
pub(super) const LCD_BLACK: Color32 = Color32::from_rgb(42, 69, 59);

pub(super) fn bgp_color_from_value(value: u8) -> Color32 {
    match value {
        0b00 => LCD_WHITE,
        0b01 => LCD_LIGHT_GRAY,
        0b10 => LCD_GRAY,
        0b11 => LCD_BLACK,
        _ => unreachable!(),
    }
}