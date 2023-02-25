use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use eframe::{
    egui::{
        menu, Button, CentralPanel, CollapsingHeader, Context, Key, KeyboardShortcut, Modifiers,
        RichText, TextureOptions, TopBottomPanel, Window,
    },
    epaint::{Color32, ColorImage},
    App, CreationContext, Frame, Storage,
};
use egui::Grid;
use egui_extras::RetainedImage;
use hashlink::LinkedHashSet;

use crate::emulator::Emulator;
use crate::{
    cpu::registers::Flag,
    ppu::{
        color_palette::{Chocolate, Green, Monochrome, ScreenColor},
        ppu::{LCD_HEIGHT, LCD_WIDTH},
    },
    ui::memory_viewer::MemoryViewer,
};

use super::{
    control_panel::ControlPanel,
    palette_picker::{Palette, PalettePicker},
};
use crate::ui::frame_history::FrameHistory;

/// Overarching struct that handles the emulator
/// and keeps track of UI elements since we are using
/// an immediate mode GUI.
pub struct Kevboy {
    emulator: Emulator,
    history: FrameHistory,

    frame_buffer: Vec<Color32>,
    raw_fb: Vec<Color32>,

    mem_viewer: MemoryViewer,
    control_panel: ControlPanel,
    palette_picker: PalettePicker,

    recent_roms: LinkedHashSet<PathBuf>,
    is_vram_window_open: bool,
}

/// Exposes two functions to create the overarching emulator object
impl Kevboy {
    /// Create overarching emulator object with no ROM loaded
    ///
    /// `CreationContext` is needed for its `storage`, so that we can
    /// store some local settings like controls, colors, etc.
    pub fn new(cc: &CreationContext) -> Self {
        Self {
            emulator: Emulator::new(),
            history: FrameHistory::default(),

            frame_buffer: [Green::WHITE].repeat(LCD_WIDTH * LCD_HEIGHT),
            raw_fb: [Green::WHITE].repeat(256 * 256),

            mem_viewer: MemoryViewer::new(),
            control_panel: ControlPanel::new(cc),
            palette_picker: PalettePicker::new(cc),

            recent_roms: eframe::get_value::<LinkedHashSet<_>>(cc.storage.unwrap(), "recent_roms")
                .unwrap_or_default(),
            is_vram_window_open: false,
        }
    }

    /// For starting the emulator from the command line
    pub fn with_rom(rom: &[u8], cc: &CreationContext) -> Self {
        let mut emulator = Emulator::new();
        emulator.load_rom(rom);

        Self {
            emulator,
            history: FrameHistory::default(),

            frame_buffer: [Green::WHITE].repeat(LCD_WIDTH * LCD_HEIGHT),
            raw_fb: [Green::WHITE].repeat(256 * 256),

            mem_viewer: MemoryViewer::new_with_memory(rom, true),
            control_panel: ControlPanel::new(cc),
            palette_picker: PalettePicker::new(cc),

            recent_roms: eframe::get_value::<LinkedHashSet<_>>(cc.storage.unwrap(), "recent_roms")
                .unwrap_or_default(),
            is_vram_window_open: false,
        }
    }
}

impl App for Kevboy {
    /// Called on shutdown and regular intervals, uses local filesystem or local storage (web)
    ///
    /// We save colors, controls and recently opened ROMs.
    fn save(&mut self, _storage: &mut dyn Storage) {
        eframe::set_value(_storage, "colors", &self.palette_picker.colors);
        eframe::set_value(_storage, "dir_controls", &self.control_panel.direction_keys);
        eframe::set_value(_storage, "action_controls", &self.control_panel.action_keys);
        eframe::set_value(_storage, "recent_roms", &self.recent_roms);
    }

    /// UI declarations and functionality, called every frame and also runs the emulator
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.history.update(ctx, frame);
        frame.set_window_title(&format!("Kevboy ({} fps)", self.history.fps()));

        let image = RetainedImage::from_color_image(
            "frame",
            ColorImage {
                size: [LCD_WIDTH, LCD_HEIGHT],
                pixels: self.frame_buffer.clone(),
            },
        )
        .with_options(TextureOptions::NEAREST);

        // ----------------------------------
        //      Start of UI declarations
        // ----------------------------------

        TopBottomPanel::top("menu").show(ctx, |root| {
            menu::bar(root, |ui| {
                ui.menu_button("File", |ui| {
                    // Opens File Dialog filtered for the most common extensions.
                    // Inserts path into `recent_roms` and saves it into local storage.
                    // Then, loads the rom into the emulator and inits the memory viewer.
                    if ui.button("Open ROM").clicked() {
                        let file = rfd::FileDialog::new()
                            .add_filter("Game Boy ROM", &["gb", "bin", "gbc"])
                            .pick_file();

                        if let Some(path) = file {
                            let rom = fs::read(path.clone()).expect("ROM wasn't loaded correctly!");
                            self.recent_roms.insert(path);

                            if let Some(storage) = frame.storage_mut() {
                                eframe::set_value(storage, "recent_roms", &self.recent_roms);
                                storage.flush();
                            }

                            self.emulator.load_rom(&rom);
                            self.mem_viewer = MemoryViewer::new_with_memory(&rom, true);
                        }

                        ui.close_menu();
                    }

                    // Iterates through a copy of `recent_roms` and generates menu buttons from it
                    // so that all the most recently loaded roms are there. Only displays the file name, not the full path.
                    // Loads the emulator and memory viewer upon clicking a rom.
                    ui.menu_button("Open recent ROMs", |ui| {
                        for rom_path in self.recent_roms.clone().iter().rev() {
                            if ui.button(rom_path.file_name().unwrap().to_str().unwrap()).clicked() {
                                let rom = fs::read(rom_path).expect("ROM wasn't loaded correctly!");
                                self.recent_roms.to_back(rom_path);

                                self.emulator.load_rom(&rom);
                                self.mem_viewer = MemoryViewer::new_with_memory(&rom, true);

                                ui.close_menu();
                            }
                        }
                    });

                    ui.separator();

                    // Load save file and restarts the game,
                    // this is only enabled when a game is already loaded,
                    // so that we can determine what game the save file belongs to.
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

                    // Store save file, only enabled when game is already loaded.
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

                // Options for changing controls and color palettes.
                // Both may open a new window and will save to local storage.
                ui.menu_button("Options", |ui| {
                    if ui.button("Controls").clicked() {
                        self.control_panel.open = !self.control_panel.open;
                    };

                    ui.menu_button("Change palette",|ui| {
                        if ui.radio_value(&mut self.palette_picker.current_palette, Palette::Monochrome(Monochrome), "Monochrome").clicked() {
                            self.palette_picker.change_colors(&Monochrome::BLACK, &Monochrome::GRAY, &Monochrome::LIGHT_GRAY, &Monochrome::WHITE);
                        }
                        if ui.radio_value(&mut self.palette_picker.current_palette, Palette::Green(Green), "LCD Green").clicked() {
                            self.palette_picker.change_colors(&Green::BLACK, &Green::GRAY, &Green::LIGHT_GRAY, &Green::WHITE);
                        }
                        if ui.radio_value(&mut self.palette_picker.current_palette, Palette::Chocolate(Chocolate), "Chocolate").clicked() {
                            self.palette_picker.change_colors(&Chocolate::BLACK, &Chocolate::GRAY, &Chocolate::LIGHT_GRAY, &Chocolate::WHITE);
                        }
                        if ui.radio_value(&mut self.palette_picker.current_palette, Palette::Custom, "Custom").clicked() {
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

                    // Cartridge Info
                    CollapsingHeader::new("Cartridge Information")
                        .default_open(true)
                        .show(ui, |ui| {
                            Grid::new("cartridge").striped(true).show(ui, |ui| {
                                ui.label("Title: ");
                                ui.label(&self.emulator.bus.cartridge.title);
                                ui.end_row();

                                ui.label("MBC: ");
                                ui.label(self.emulator.get_full_mbc_title().unwrap_or_default());
                                ui.end_row();

                                ui.label("Destination: ");
                                ui.label(self.emulator.get_destination_code().unwrap_or_default());
                                ui.end_row();

                                ui.label("ROM size: ");
                                ui.label(format!(
                                    "{} KiB",
                                    self.emulator.get_rom_size().unwrap_or_default()
                                ));
                                ui.end_row();

                                ui.label("RAM size: ");
                                ui.label(format!(
                                    "{} KiB",
                                    self.emulator.get_ram_size().unwrap_or_default()
                                ));
                                ui.end_row();
                            });
                        });
                });
            });
        });

        // ------------------------------------
        //    Handle open state of windows
        // ------------------------------------

        // Change and customize controls in this window
        if self.control_panel.open {
            let mut control_panel_open = self.control_panel.open;
            Window::new("⌨ Controls")
                .open(&mut control_panel_open)
                .resizable(false)
                .show(ctx, |ui| {
                    self.control_panel.show(ctx, ui, frame);
                });
            self.control_panel.open &= control_panel_open;
        }

        // Change and customize the color palette of the Game Boy
        if self.palette_picker.open {
            let mut palette_window_open = self.palette_picker.open;
            Window::new("🎨 Palettes")
                .open(&mut palette_window_open)
                .resizable(false)
                .show(ctx, |ui| {
                    self.palette_picker.show(ui, frame);
                });
            self.palette_picker.open &= palette_window_open;
        }

        if self.mem_viewer.open {
            let mut mem_viewer_open = self.mem_viewer.open;
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
                        ColorImage {
                            size: [256, 256],
                            pixels: self.raw_fb.clone(),
                        },
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

/// Second impl block for the run function
impl Kevboy {
    fn run(&mut self, ctx: &Context) {
        while self.emulator.cycle_count < 17_556 {
            self.emulator.bus.joypad.tick(
                ctx,
                &mut self.emulator.bus.interrupt_handler,
                &self.control_panel.action_keys,
                &self.control_panel.direction_keys,
            );

            self.emulator.cycle_count += self.emulator.step() as u16;
        }

        // Normal frame buffer for frontend, gets swapped for double buffering
        self.frame_buffer = self
            .emulator
            .bus
            .ppu
            .ui_frame_buffer
            .iter()
            .map(|c| match *c {
                ScreenColor::White => self.palette_picker.colors["White"],
                ScreenColor::LightGray => self.palette_picker.colors["Light Gray"],
                ScreenColor::Gray => self.palette_picker.colors["Gray"],
                ScreenColor::Black => self.palette_picker.colors["Black"],
            })
            .collect();

        // Raw background tile map for debugging
        self.raw_fb = self
            .emulator
            .bus
            .ppu
            .raw_frame
            .iter()
            .map(|c| match *c {
                ScreenColor::White => self.palette_picker.colors["White"],
                ScreenColor::LightGray => self.palette_picker.colors["Light Gray"],
                ScreenColor::Gray => self.palette_picker.colors["Gray"],
                ScreenColor::Black => self.palette_picker.colors["Black"],
            })
            .collect();

        self.emulator.cycle_count = 0;
        self.emulator.bus.joypad.reset_pressed_keys();
    }
}
