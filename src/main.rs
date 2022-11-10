#![allow(dead_code)] // TODO

use ui::ui::Kevboy;

mod cartridge;
mod cpu;
mod emulator;
mod input;
mod mmu;
mod ppu;
mod ui;

const LCD_WIDTH: usize = 160;
const LCD_HEIGHT: usize = 144;

fn main() {
    // simple_logger::init().unwrap();
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Kevboy-rs",
        native_options,
        Box::new(|_| Box::new(Kevboy::default())),
    );
}
