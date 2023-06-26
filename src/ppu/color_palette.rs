use eframe::epaint::Color32;

/// Palette enum with the according palette register as the associated value.
///
/// In CGB mode, `BGP` represents a value between 0 and 7.
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
    FullColor(Color32),
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
pub(super) fn convert_to_color(value: u8, palette: Palette, cgb: bool, cram: &[u8]) -> ScreenColor {
    match palette {
        Palette::BGP(bgp) if !cgb => match value {
            0b00 => color_from_value(bgp & 0b11),
            0b01 => color_from_value((bgp & 0b1100) >> 2),
            0b10 => color_from_value((bgp & 0b110000) >> 4),
            0b11 => color_from_value((bgp & 0b11000000) >> 6),
            _ => unreachable!(),
        },
        Palette::BGP(bgp) if cgb => {
            let palette = (bgp * 8 + value * 2) as usize;
            let color_bytes = u16::from_le_bytes([cram[palette], cram[palette + 1]]);
            ScreenColor::FullColor(rgb555_to_color(color_bytes))
        }
        Palette::OBP(obp) if !cgb => match value {
            0b01 => color_from_value((obp & 0b1100) >> 2),
            0b10 => color_from_value((obp & 0b110000) >> 4),
            0b11 => color_from_value((obp & 0b11000000) >> 6),
            _ => unreachable!(),
        },
        Palette::OBP(obp) if cgb => {
            if value == 0 {
                return ScreenColor::FullColor(Color32::TRANSPARENT);
            }

            let palette = (obp * 8 + value * 2) as usize;
            let color_bytes = u16::from_le_bytes([cram[palette], cram[palette + 1]]);
            ScreenColor::FullColor(rgb555_to_color(color_bytes))
        }
        _ => unreachable!(),
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

fn rgb555_to_color(rgb: u16) -> Color32 {
    let red = (rgb & 0x1F) as u8;
    let green = ((rgb >> 5) & 0x1F) as u8;
    let blue: u8 = ((rgb >> 10) & 0x1F) as u8;

    Color32::from_rgb(
        (red << 3) | (red >> 2),
        (green << 3) | (green >> 2),
        (blue << 3) | (blue >> 2),
    )
}
