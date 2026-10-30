#![allow(dead_code)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]
#![windows_subsystem = "windows"]

use anyhow::Result;
use eframe::icon_data::from_png_bytes;
use egui::{FontDefinitions, ViewportBuilder};
use egui_extras::install_image_loaders;
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
    let icon_data = from_png_bytes(include_bytes!("../icon/icon.png"))?;
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_icon(icon_data),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Kevboy",
        native_options,
        Box::new(|cc| {
            let mut fonts = FontDefinitions::default();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
            cc.egui_ctx.set_fonts(fonts);

            install_image_loaders(&cc.egui_ctx);

            // Read in rom per command line
            let kevboy = match std::env::args().nth(1) {
                Some(rom) => Kevboy::with_rom(&std::fs::read(rom).unwrap(), cc),
                None => Kevboy::new(cc),
            };

            Ok(Box::new(kevboy))
        }),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))
}
