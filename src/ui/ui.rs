use std::{
    fs::{self, File},
    io::Write,
};

use eframe::{
    egui::{
        menu, Button, CentralPanel, CollapsingHeader, Context, Key,
        KeyboardShortcut, Modifiers, RadioButton, RichText, TextEdit, TextureOptions,
        TopBottomPanel, Window,
    },
    epaint::{Color32, ColorImage},
    App, Frame,
};
use egui_extras::RetainedImage;

use crate::emulator::Emulator;
use crate::{
    cpu::registers::Flag,
    ppu::ppu::{LCD_HEIGHT, LCD_WIDTH},
    ui::memory_viewer::MemoryViewer,
};

use super::{control_panel::ControlPanel, palette_picker::PalettePicker};
use crate::ui::frame_history::FrameHistory;

pub struct Kevboy {
    emulator: Emulator,
    history: FrameHistory,

    frame_buffer: Vec<u8>,
    raw_fb: Vec<u8>,

    mem_viewer: MemoryViewer,
    control_panel: ControlPanel,
    palette_picker: PalettePicker,

    is_vram_window_open: bool,
}

impl Default for Kevboy {
    fn default() -> Self {
        Self {
            emulator: Emulator::new(),
            history: FrameHistory::default(),
            
            frame_buffer: [127, 134, 15, 255].repeat(LCD_WIDTH * LCD_HEIGHT),
            raw_fb: [127, 134, 15, 255].repeat(256 * 256),

            mem_viewer: MemoryViewer::new(),
            control_panel: ControlPanel::default(),
            palette_picker: PalettePicker::default(),

            is_vram_window_open: false,
        }
    }
}

impl App for Kevboy {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.history.update(ctx, frame);
        frame.set_window_title(&format!("Kevboy ({} fps)", self.history.fps().trunc()));

        let image = RetainedImage::from_color_image(
            "frame",
            ColorImage::from_rgba_unmultiplied([LCD_WIDTH, LCD_HEIGHT], &self.frame_buffer),
        )
        .with_options(TextureOptions::NEAREST);

        // ----------------------------------
        //      Start of UI declarations
        // ----------------------------------

        TopBottomPanel::top("menu").show(ctx, |root| {
            menu::bar(root, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open ROM").clicked() {
                        let file = rfd::FileDialog::new()
                            .add_filter("Gameboy ROM", &["gb", "bin", "gbc"])
                            .pick_file();

                        if let Some(path) = file {
                            let rom = fs::read(path).expect("ROM wasn't loaded correctly!");
                            self.emulator.load_rom(&rom);
                            self.mem_viewer = MemoryViewer::new_with_memory(&rom, true);
                        }

                        ui.close_menu();
                    }

                    ui.separator();

                    // Load save file
                    if ui
                        .add_enabled(
                            !self.emulator.rom.is_empty(),
                            Button::new("Load Save").shortcut_text(
                                ctx.format_shortcut(&KeyboardShortcut::new(
                                    Modifiers::CTRL,
                                    Key::L,
                                )),
                            ),
                        )
                        .clicked()
                    {
                        let file = rfd::FileDialog::new()
                            .add_filter("Save file", &["sav"])
                            .pick_file();

                        if let Some(path) = file {
                            let save_file = fs::read(path).expect("Save file wasn't loaded correctly!");

                            // restart ROM so that the save can be applied before it's too late
                            self.emulator.load_rom(&self.emulator.rom.clone());
                            self.emulator.bus.cartridge.load_sram(&save_file);
                        }
                    }

                    // Store save file
                    if ui
                        .add_enabled(
                            !self.emulator.rom.is_empty(),
                            Button::new("Store Save").shortcut_text(
                                ctx.format_shortcut(&KeyboardShortcut::new(
                                    Modifiers::CTRL,
                                    Key::S,
                                )),
                            ),
                        )
                        .clicked()
                    {
                        let file = rfd::FileDialog::new().add_filter("Save file", &["sav"]).save_file();
                        let sram = self.emulator.bus.cartridge.dump_sram();

                        if let Some(f) = file {
                            let save_file = File::create(f);
                            if let Ok(mut sf) = save_file {
                                if let Some(sram) = sram {
                                    sf.write_all(&sram).unwrap();
                                } else {
                                    rfd::MessageDialog::new().set_title("No saving was done!")
                                        .set_description("Nothing was saved as this cartridge does not support external RAM.").show();
                                }
                            }
                        }
                    }
                });

                ui.menu_button("Options", |ui| {
                    if ui.button("Controls").clicked() {
                        self.control_panel.open = !self.control_panel.open;
                    };

                    ui.menu_button("Change palette",|ui| {
                        ui.add(RadioButton::new(false, "Monochrome"));
                        ui.add(RadioButton::new(true, "LCD Green"));
                        ui.add(RadioButton::new(false, "Chocolate"));
                        if ui.add(RadioButton::new(false, "Custom...")).clicked() {
                            self.palette_picker.open = !self.palette_picker.open;
                        }
                    });
                });

                ui.menu_button("Debug", |ui| {
                    if ui.button("Show memory (hex)").clicked() {
                        self.mem_viewer.open = !self.mem_viewer.open;
                    }
                    if ui.button("Open VRAM viewer").clicked() {
                        self.is_vram_window_open = !self.is_vram_window_open;
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
                                    .code_editor(),
                            );
                        });
                });
            });
        });

        // ------------------------------------
        //    Handle open state of windows
        // ------------------------------------

        // Change and customize controls in this window
        if self.control_panel.open {
            let mut control_panel_open = self.control_panel.open.clone();
            Window::new("⌨ Controls")
                .open(&mut control_panel_open)
                .resizable(false)
                .show(ctx, |ui| {
                    self.control_panel.show(ctx, ui);
                });
            self.control_panel.open = control_panel_open;
        }

        // Change and customize the color palette of the Game Boy
        if self.palette_picker.open {
            let mut palette_window_open = self.palette_picker.open.clone();
            Window::new("🎨 Palettes")
                .open(&mut palette_window_open)
                .resizable(false)
                .show(ctx, |ui| {
                    self.palette_picker.show(ui);
                });
            self.palette_picker.open = palette_window_open;
        }

        if self.mem_viewer.open {
            let mut mem_viewer_open = self.mem_viewer.open.clone();
            Window::new("💾 Memory")
                .open(&mut mem_viewer_open)
                .show(ctx, |ui| {
                    self.mem_viewer.show(ui);
                });
            self.mem_viewer.open = mem_viewer_open;
        }

        if self.is_vram_window_open {
            Window::new("🖼 BG Map")
                .open(&mut self.is_vram_window_open)
                .show(ctx, |ui| {
                    let image = RetainedImage::from_color_image(
                        "vram",
                        ColorImage::from_rgba_unmultiplied([256, 256], &self.raw_fb),
                    )
                    .with_options(TextureOptions::NEAREST);

                    // image.show_scaled(ui, 3.0);
                    image.show_size(ui, ui.available_size());
                });
        }

        // ----------------------------------
        //      End of UI declarations
        // ----------------------------------

        if !self.emulator.rom.is_empty() {
            self.run(ctx);
            ctx.request_repaint();
        }
    }
}

impl Kevboy {
    pub fn new(rom: &[u8]) -> Self {
        let mut emulator = Emulator::new();
        emulator.load_rom(rom);

        Self {
            emulator,
            mem_viewer: MemoryViewer::new_with_memory(rom, true),
            ..Default::default()
        }
    }

    fn run(&mut self, ctx: &Context) {
        while self.emulator.cycle_count < 17_556 {
            self.emulator
                .bus
                .joypad
                .tick(ctx, &mut self.emulator.bus.interrupt_handler);

            self.emulator.cycle_count += self.emulator.step() as u16;
        }

        // Normal frame buffer for frontend, gets swapped for double buffering
        self.frame_buffer = self
            .emulator
            .bus
            .ppu
            .ui_frame_buffer
            .iter()
            .flat_map(|c| c.to_array())
            .collect();

        // Raw background tile map for debugging
        self.raw_fb = self
            .emulator
            .bus
            .ppu
            .raw_frame
            .iter()
            .flat_map(|c| c.to_array())
            .collect();

        self.emulator.cycle_count = 0;
        self.emulator.bus.joypad.reset_pressed_keys();
    }
}
