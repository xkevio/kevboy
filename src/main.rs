#![allow(dead_code)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]
#![windows_subsystem = "windows"]

use anyhow::Result;
use eframe::IconData;
use egui::FontDefinitions;
use image::{codecs::png::PngDecoder, DynamicImage};
use ui::Kevboy;

#[path = "apu/apu.rs"]
mod apu;
mod cartridge;
#[path = "cpu/cpu.rs"]
mod cpu;
mod emulator;
mod input;
mod mmu;
#[path = "ppu/ppu.rs"]
mod ppu;
#[path = "ui/ui.rs"]
mod ui;

fn main() -> Result<()> {
    let icon = include_bytes!("../icon/icon.png");
    let icon_data = DynamicImage::from_decoder(PngDecoder::new(&icon[..])?)?;

    let native_options = eframe::NativeOptions {
        vsync: false,
        centered: true,
        icon_data: Some(IconData {
            rgba: icon_data.as_bytes().to_vec(),
            width: 256,
            height: 256,
        }),
        ..Default::default()
    };

    eframe::run_native(
        "Kevboy",
        native_options,
        Box::new(|cc| {
            let mut fonts = FontDefinitions::default();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
            cc.egui_ctx.set_fonts(fonts);

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
