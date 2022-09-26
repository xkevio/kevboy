#![allow(dead_code)] // TODO

use eframe::epaint::Vec2;
use ui::Kevboy;

mod bus;
mod cpu;
mod memory_viewer;
mod opcode;
mod registers;
mod ui;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

const BOOT_ROM: &[u8; 1048576] = include_bytes!("red_test.gb");

fn main() {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(1400.0, 720.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Kevboy-rs",
        native_options,
        Box::new(|_| Box::new(Kevboy::default())),
    );
}
