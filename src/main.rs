#![allow(dead_code)]
#![windows_subsystem = "windows"]

use egui::FontDefinitions;
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
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "Kevboy",
                web_options,
                Box::new(|cc| {
                    let mut fonts = FontDefinitions::default();
                    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
                    cc.egui_ctx.set_fonts(fonts);

                    Box::new(Kevboy::new(cc))
                }),
            )
            .await
            .expect("failed to start eframe");
    });
}
