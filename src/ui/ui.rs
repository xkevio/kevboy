use std::{
    fs,
    time::{Duration, Instant},
};

use eframe::{
    egui::{
        menu, CentralPanel, CollapsingHeader, RichText, TextEdit, TextStyle, TopBottomPanel, Window,
    },
    epaint::{Color32, ColorImage},
    App,
};
use egui_extras::RetainedImage;

use crate::ui::memory_viewer::MemoryViewer;
use crate::{emulator::Emulator, LCD_HEIGHT, LCD_WIDTH};

pub struct Kevboy {
    emulator: Emulator,
    cy_count: u16,
    mem_viewer: MemoryViewer,
    is_memory_window_open: bool,
    frame: Vec<u8>,
}

impl Default for Kevboy {
    fn default() -> Self {
        Self {
            emulator: Emulator::new(),
            cy_count: 0,
            mem_viewer: MemoryViewer::new(),
            is_memory_window_open: false,
            frame: [127, 134, 15, 255].repeat(LCD_WIDTH * LCD_HEIGHT),
        }
    }
}

impl App for Kevboy {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let image = RetainedImage::from_color_image(
            "frame",
            ColorImage::from_rgba_unmultiplied([LCD_WIDTH, LCD_HEIGHT], &self.frame),
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
                            let rom = fs::read(path).expect("ROM wasn't loaded correctly!");

                            self.emulator.load_rom(&rom);
                            self.mem_viewer = MemoryViewer::new_with_memory(&rom, true);
                        }

                        ui.close_menu();
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
                                image.show_scaled(ui, 3.0);
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
                                    .hint_text("Disassembly not implemented yet...")
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

        if !self.emulator.rom.is_empty() {
            while self.cy_count < 17_476 {
                self.cy_count += self.emulator.step() as u16;
            }

            let buf = self
                .emulator
                .bus
                .ppu
                .get_frame_viewport()
                .iter()
                .flat_map(|c| [c.r(), c.g(), c.b(), c.a()])
                .collect();

            self.frame = buf;

            let start = Instant::now();
            while start.elapsed() < Duration::from_micros(16667) {
                // do nothing
            }

            self.cy_count = 0;
            ctx.request_repaint();
        }
    }
}
