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

use crate::{cpu::registers::Flag, ui::memory_viewer::MemoryViewer};
use crate::{emulator::Emulator, LCD_HEIGHT, LCD_WIDTH};

use crate::ui::frame_history::FrameHistory;

pub struct Kevboy {
    emulator: Emulator,
    frame_buffer: Vec<u8>,

    // To count and calculate frames per second, smoothed
    history: FrameHistory,

    mem_viewer: MemoryViewer,
    is_memory_viewer_open: bool,
}

impl Default for Kevboy {
    fn default() -> Self {
        Self {
            emulator: Emulator::new(),
            frame_buffer: [127, 134, 15, 255].repeat(LCD_WIDTH * LCD_HEIGHT),

            history: FrameHistory::default(),

            mem_viewer: MemoryViewer::new(),
            is_memory_viewer_open: false,
        }
    }
}

impl App for Kevboy {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.history.update(ctx, frame);
        frame.set_window_title(&format!("Kevboy-rs ({} fps)", self.history.fps().trunc()));

        let image = RetainedImage::from_color_image(
            "frame",
            ColorImage::from_rgba_unmultiplied([LCD_WIDTH, LCD_HEIGHT], &self.frame_buffer),
        )
        .with_texture_filter(eframe::egui::TextureFilter::Nearest);

        // ----------------------------------
        //      Start of UI declarations
        // ----------------------------------

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
                        self.is_memory_viewer_open = !self.is_memory_viewer_open;
                    }
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Game
                CollapsingHeader::new("Game")
                    .default_open(true)
                    .show(ui, |ui| {
                        image.show_scaled(ui, 3.0);
                    });

                // Registers + Instructions
                ui.vertical(|ui| {
                    // Registers
                    CollapsingHeader::new("Registers")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("AF:\nBC:\nDE:\nHL:\n\nSP:\nPC:")
                                        .strong()
                                        .monospace()
                                        .color(Color32::GRAY),
                                );

                                ui.label(
                                    RichText::new(format!(
                                        "{:#06X}\n{:#06X}\n{:#06X}\n{:#06X}\n\n{:#06X}\n{:#06X}",
                                        self.emulator.cpu.registers.get_af(),
                                        self.emulator.cpu.registers.get_bc(),
                                        self.emulator.cpu.registers.get_de(),
                                        self.emulator.cpu.registers.get_hl(),
                                        self.emulator.cpu.registers.SP,
                                        self.emulator.cpu.registers.PC
                                    ))
                                    .strong()
                                    .monospace()
                                    .color(Color32::GOLD),
                                );
                            });

                            ui.label("\nFlags:\n");
                            ui.label(
                                RichText::new("Z\t\tN\t\tH\t\tC\n")
                                    .strong()
                                    .monospace()
                                    .color(Color32::GRAY),
                            );

                            ui.label(
                                RichText::new(format!(
                                    "{}\t\t{}\t\t{}\t\t{}",
                                    self.emulator.cpu.registers.get_flag(Flag::Zero) as u8,
                                    self.emulator.cpu.registers.get_flag(Flag::Substraction) as u8,
                                    self.emulator.cpu.registers.get_flag(Flag::HalfCarry) as u8,
                                    self.emulator.cpu.registers.get_flag(Flag::Carry) as u8
                                ))
                                .strong()
                                .monospace()
                                .color(Color32::GOLD),
                            );
                        });

                    ui.add_space(10.0);

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

        if self.is_memory_viewer_open {
            Window::new("Memory")
                .open(&mut self.is_memory_viewer_open)
                .show(ctx, |ui| {
                    self.mem_viewer.show(ui);
                });
        }

        // ----------------------------------
        //      End of UI declarations
        // ----------------------------------

        if !self.emulator.rom.is_empty() {
            self.run();
            ctx.request_repaint();
        }
    }
}

impl Kevboy {
    fn run(&mut self) {
        while self.emulator.cycle_count < 17_476 {
            self.emulator.cycle_count += self.emulator.step() as u16;
        }

        let buf = self
            .emulator
            .bus
            .ppu
            .frame_buffer
            .iter()
            .flat_map(|c| [c.r(), c.g(), c.b(), c.a()])
            .collect();

        self.frame_buffer = buf;

        let start = Instant::now();
        while start.elapsed() < Duration::from_micros(16667) {
            // do nothing
        }

        self.emulator.cycle_count = 0;
    }
}
