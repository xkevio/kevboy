#![allow(dead_code)] // TODO

use ui::ui::Kevboy;

mod cartridge;
mod cpu;
mod emulator;
mod input;
mod mmu;
mod ppu;
mod ui;

fn main() {
    // use native vsync for "sync to video" to keep gameboy at 60fps (technically 59.73)
    // will only work on 60Hz displays for now
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Kevboy",
        native_options,
        Box::new(|_| Box::new(Kevboy::default())),
    );
}
