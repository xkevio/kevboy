#![allow(dead_code)] // TODO

use ui::ui::Kevboy;

mod cpu;
mod emulator;
mod mmu;
mod ui;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

const TEST_ROM: &[u8; 32768] = include_bytes!("../blargg_tests/instr_timing.gb");

fn main() {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Kevboy-rs",
        native_options,
        Box::new(|_| Box::new(Kevboy::default())),
    );
}
