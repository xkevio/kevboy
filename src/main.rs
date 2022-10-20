#![allow(dead_code)] // TODO

use ui::ui::Kevboy;

// #[cfg(target_arch = "wasm32")]
// use wasm_bindgen::prelude::*;

mod cpu;
mod emulator;
mod mmu;
mod ppu;
mod ui;

const LCD_WIDTH: usize = 160;
const LCD_HEIGHT: usize = 144;

/// Call this once from the HTML.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();
    eframe::start_web(
        "Kevboy-rs", // hardcode it
        web_options,
        Box::new(|_| Box::new(Kevboy::default())),
    )
    .expect("failed to start eframe");
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // simple_logger::init().unwrap();
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Kevboy-rs",
        native_options,
        Box::new(|_| Box::new(Kevboy::default())),
    );
}
