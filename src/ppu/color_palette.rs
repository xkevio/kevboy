use eframe::epaint::Color32;

/// Palette enum with the according palette register as the associated value.
///
/// `OBP` shall be used for both obp0 and obp1 as long as the correct palette is passed.
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy)]
pub enum Palette {
    BGP(u8),
    OBP(u8),
}

/// Abstraction for screen color, does not hold color information on its own.
///
/// Gets transformed into chosen color palette by the UI.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScreenColor {
    White,
    LightGray,
    Gray,
    Black,
}

// Pre-defined color palettes based on associated constants
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Monochrome;
impl Monochrome {
    pub const WHITE: Color32 = Color32::WHITE;
    pub const LIGHT_GRAY: Color32 = Color32::LIGHT_GRAY;
    pub const GRAY: Color32 = Color32::GRAY;
    pub const BLACK: Color32 = Color32::BLACK;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Green;
impl Green {
    pub const WHITE: Color32 = Color32::from_rgb(127, 134, 15);
    pub const LIGHT_GRAY: Color32 = Color32::from_rgb(87, 124, 68);
    pub const GRAY: Color32 = Color32::from_rgb(54, 93, 72);
    pub const BLACK: Color32 = Color32::from_rgb(42, 69, 59);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chocolate;
impl Chocolate {
    pub const WHITE: Color32 = Color32::from_rgb(255, 228, 194);
    pub const LIGHT_GRAY: Color32 = Color32::from_rgb(220, 164, 86);
    pub const GRAY: Color32 = Color32::from_rgb(169, 96, 76);
    pub const BLACK: Color32 = Color32::from_rgb(66, 41, 54);
}

/// Takes the color value byte and transforms into the correct color based on the palette register
pub(super) fn convert_to_color(value: u8, palette: Palette) -> ScreenColor {
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

fn color_from_value(value: u8) -> ScreenColor {
    match value {
        0b00 => ScreenColor::White,
        0b01 => ScreenColor::LightGray,
        0b10 => ScreenColor::Gray,
        0b11 => ScreenColor::Black,
        _ => unreachable!(),
    }
}
