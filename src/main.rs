#![allow(dead_code)]
// #![windows_subsystem = "windows"]

use anyhow::Result;
use eframe::IconData;
use image::{codecs::png::PngDecoder, DynamicImage};
use ui::ui::Kevboy;

mod apu;
mod cartridge;
mod cpu;
mod emulator;
mod input;
mod mmu;
mod ppu;
mod ui;

fn main() -> Result<()> {
    // Use native vsync for "sync to video" to keep gameboy at 60fps (technically 59.73)
    // Will only work on 60Hz displays for now
    let mut native_options = eframe::NativeOptions::default();

    let icon = include_bytes!("../icon/icon.png");
    let icon_data = DynamicImage::from_decoder(PngDecoder::new(&icon[..])?)?;

    // VSync needs to be disabled for unthrottling!
    native_options.vsync = false;
    native_options.icon_data = Some(IconData {
        rgba: icon_data.as_bytes().to_vec(),
        width: 256,
        height: 256,
    });
    native_options.centered = true;

    eframe::run_native(
        "Kevboy",
        native_options,
        Box::new(|cc| {
            // Read in rom per command line
            let kevboy = match std::env::args().nth(1) {
                Some(rom) => Kevboy::with_rom(&std::fs::read(rom).unwrap(), cc),
                None => Kevboy::new(cc),
            };
            Box::new(kevboy)
        }),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))
}
