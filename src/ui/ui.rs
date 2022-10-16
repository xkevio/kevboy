use std::time::{Duration, Instant};

use eframe::{
    egui::{
        menu, CentralPanel, CollapsingHeader, RichText, TextEdit, TextStyle, TopBottomPanel, Window,
    },
    epaint::{Color32, ColorImage},
    App,
};
use egui_extras::RetainedImage;

use crate::WIDTH;
use crate::{emulator::Emulator, HEIGHT};
use crate::{ui::memory_viewer::MemoryViewer, TEST_ROM};

pub struct Kevboy<'a> {
    emulator: Emulator,
    cy_count: u16,
    mem_viewer: MemoryViewer<'a>,
    is_memory_window_open: bool,
    frame: Vec<u8>,
}

impl<'a> Default for Kevboy<'a> {
    fn default() -> Self {
        Self {
            emulator: Emulator::new(TEST_ROM),
            cy_count: 0,
            mem_viewer: MemoryViewer::new(TEST_ROM, true),
            is_memory_window_open: false,
            frame: [224, 248, 208, 255].repeat(256 * 256),
        }
    }
}

impl<'a> App for Kevboy<'a> {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let image = RetainedImage::from_color_image(
            "frame",
            ColorImage::from_rgba_unmultiplied([256, 256], &self.frame),
        )
        .with_texture_filter(eframe::egui::TextureFilter::Nearest);

        TopBottomPanel::top("menu").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open ROM").clicked() {
                        let file = rfd::FileDialog::new()
                            .add_filter("Gameboy ROM", &["gb", "bin"])
                            .pick_file();

                        if let Some(path) = file {
                            println!("{}", path.to_str().unwrap());
                        }
                    }
                });

                ui.menu_button("Options", |_ui| {});

                ui.menu_button("Debug", |ui| {
                    if ui.button("Show memory (hex)").clicked() {
                        self.is_memory_window_open = !self.is_memory_window_open;
                    }
                });
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
                                image.show_scaled(ui, 1.5);
                            });

                        // Registers
                        CollapsingHeader::new("Registers")
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new("AF:\nBC:\nDE:\nHL:\n\nSP:\nPC:")
                                        .strong()
                                        .monospace()
                                        .color(Color32::GRAY),
                                );
                                ui.label("\nFlags:\n");
                                ui.label(
                                    RichText::new("Z\t\tN\t\tH\t\tC\n")
                                        .strong()
                                        .monospace()
                                        .color(Color32::GRAY),
                                );
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
            });
        });

        if self.is_memory_window_open {
            Window::new("Memory")
                .open(&mut self.is_memory_window_open)
                .show(ctx, |ui| {
                    self.mem_viewer.show(ui);
                });
        }

        // println!("{:?}", Instant::now());
        while self.cy_count < 17_476 {
            self.cy_count += self.emulator.step() as u16;
        }

        let new_buffer: Vec<u8> = self
            .emulator
            .bus
            .ppu
            .frame_buffer
            .iter()
            .map(|c| [c.r(), c.g(), c.b(), c.a()])
            .flatten()
            .collect();

        self.frame = new_buffer;

        let start = Instant::now();
        while start.elapsed() < Duration::from_micros(16667) {
            // do nothing
        }

        self.cy_count = 0;

        ctx.request_repaint();
    }
}
