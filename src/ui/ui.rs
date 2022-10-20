use std::{
    fs,
    time::{Duration, Instant}, sync::mpsc::{Sender, Receiver},
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

use super::frame_history::FrameHistory;

pub struct Kevboy {
    emulator: Emulator,
    frame: Vec<u8>,
    history: FrameHistory,
    cy_count: u16,
    mem_viewer: MemoryViewer,
    is_memory_window_open: bool,
    channel_test: (Sender<Vec<u8>>, Receiver<Vec<u8>>)
}

impl Default for Kevboy {
    fn default() -> Self {
        Self {
            emulator: Emulator::new(),
            frame: [127, 134, 15, 255].repeat(LCD_WIDTH * LCD_HEIGHT),
            history: FrameHistory::default(),
            cy_count: 0,
            mem_viewer: MemoryViewer::new(),
            is_memory_window_open: false,
            channel_test: std::sync::mpsc::channel(),
        }
    }
}

impl App for Kevboy {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.history.update(ctx, _frame);
            _frame.set_window_title(&format!("Kevboy-rs ({} fps)", self.history.fps().trunc()));
        }

        let image = RetainedImage::from_color_image(
            "frame",
            ColorImage::from_rgba_unmultiplied([LCD_WIDTH, LCD_HEIGHT], &self.frame),
        )
        .with_texture_filter(eframe::egui::TextureFilter::Nearest);

        #[cfg(target_arch = "wasm32")]
        loop {
            match self.channel_test.1.try_recv() {
                Ok(msg) => {
                    // let string = format!("{:?}", &msg);
                    // web_sys::console::log_1(&string.into());
                    self.emulator.load_rom(&msg);
                    self.mem_viewer = MemoryViewer::new_with_memory(&msg, true);

                    // TODO: have to move mouse at first load
                }
                Err(_) => break
            }
        }

        TopBottomPanel::top("menu").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open ROM").clicked() {
                        #[cfg(not(target_arch = "wasm32"))]
                        let file = rfd::FileDialog::new()
                            .add_filter("Gameboy ROM", &["gb", "bin"])
                            .pick_file();

                        #[cfg(target_arch = "wasm32")]
                        {
                            let async_file = rfd::AsyncFileDialog::new()
                                .add_filter("Gameboy ROM", &["gb", "bin"])
                                .pick_file();

                            let sender = self.channel_test.0.clone();

                            let file_future = async move {
                                let file = async_file.await;

                                if let Some(path) = file {
                                    let rom = path.read().await;
                                    sender.send(rom).ok();
                                }
                            };

                            wasm_bindgen_futures::spawn_local(file_future);
                        }

                        #[cfg(not(target_arch = "wasm32"))]
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

            #[cfg(not(target_arch = "wasm32"))]
            let start = Instant::now();

            #[cfg(target_arch = "wasm32")]
            let start = wasm_timer::Instant::now();

            while start.elapsed() < Duration::from_micros(16667) {
                // do nothing
            }

            self.cy_count = 0;
            ctx.request_repaint();
        }
    }
}
