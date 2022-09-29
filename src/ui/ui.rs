use std::{
    io::StdoutLock,
    time::{Duration, Instant},
};

use eframe::{
    egui::{menu, CentralPanel, CollapsingHeader, TextEdit, TextStyle, TopBottomPanel},
    epaint::ColorImage,
    App,
};
use egui_extras::RetainedImage;

use crate::WIDTH;
use crate::{emulator::Emulator, HEIGHT};
use crate::{ui::memory_viewer::MemoryViewer, BOOT_ROM};

pub struct Kevboy<'a> {
    emulator: Emulator,
    cy_count: u16,
    mem_viewer: MemoryViewer<'a>,
    lock: StdoutLock<'a>,
}

impl<'a> Default for Kevboy<'a> {
    fn default() -> Self {
        Self {
            emulator: Emulator::new(BOOT_ROM),
            cy_count: 0,
            mem_viewer: MemoryViewer::new(BOOT_ROM, false),
            lock: std::io::stdout().lock(),
        }
    }
}

impl<'a> App for Kevboy<'a> {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let test_buffer = [224, 248, 208, 255].repeat(WIDTH * HEIGHT);

        let image = RetainedImage::from_color_image(
            "test",
            ColorImage::from_rgba_unmultiplied([WIDTH, HEIGHT], &test_buffer),
        )
        .with_texture_filter(eframe::egui::TextureFilter::Nearest);

        TopBottomPanel::top("menu").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open ROM").clicked() {
                        println!("Open ROM clicked!");

                        let file = rfd::FileDialog::new()
                            .add_filter("Gameboy ROM", &["gb", "bin"])
                            .pick_file();

                        if let Some(path) = file {
                            println!("{}", path.to_str().unwrap());
                        }
                    }
                });

                ui.menu_button("Options", |_ui| {});

                ui.separator();

                if ui.button("Start BOOT ROM").clicked() {
                    println!("Booting up!");
                };
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    // Game + Registers
                    ui.horizontal(|ui| {
                        // Game
                        CollapsingHeader::new("Game")
                            .default_open(true)
                            .show(ui, |ui| {
                                image.show_scaled(ui, 3.0);
                            });

                        // Registers
                        CollapsingHeader::new("Registers")
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.add(TextEdit::multiline(&mut "AF:\nBC:\nDE:\nHL:\n\nSP:\nPC:"));
                                ui.label("Flags:");
                                ui.add(TextEdit::multiline(
                                    &mut "F:\n\nZero:\nSubstraction:\nHalf-Carry:\nCarry:\n",
                                ));
                            });
                    });

                    // Instructions / Disassembly
                    CollapsingHeader::new("Instructions")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.add(
                                TextEdit::multiline(&mut "")
                                    .hint_text("No ROM loaded")
                                    .font(TextStyle::Monospace),
                            );
                        });
                });

                // Memory
                CollapsingHeader::new("Memory")
                    .default_open(true)
                    .show(ui, |ui| {
                        self.mem_viewer.show(ui);
                    });
            });
        });

        // println!("{:?}", Instant::now());
        while self.cy_count < 17_476 {
            self.cy_count += self.emulator.step(&mut self.lock) as u16;
        }

        let start = Instant::now();

        while start.elapsed() < Duration::from_micros(16667) {
            // do nothing
        }

        self.cy_count = 0;

        ctx.request_repaint();
    }
}
