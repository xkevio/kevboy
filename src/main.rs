#![allow(dead_code)]
// #![windows_subsystem = "windows"]

#[cfg(not(target_arch = "wasm32"))]
use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
use eframe::IconData;
#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "Kevboy", // hardcode it
            web_options,
            Box::new(|cc| Box::new(Kevboy::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<()> {
    // Use native vsync for "sync to video" to keep gameboy at 60fps (technically 59.73)
    // Will only work on 60Hz displays for now
    let mut native_options = eframe::NativeOptions::default();

    let icon = include_bytes!("../icon/icon.png");
    let icon_data = DynamicImage::from_decoder(PngDecoder::new(&icon[..])?)?;

    native_options.icon_data = Some(IconData {
        rgba: icon_data.as_bytes().to_vec(),
        width: 256,
        height: 256,
    });

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
