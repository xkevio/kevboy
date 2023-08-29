use std::{
    path::PathBuf,
    sync::atomic::Ordering,
    sync::mpsc::{Receiver, Sender},
};

#[cfg(not(target_arch = "wasm32"))]
use std::fs::{self, File};
#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;

use eframe::{
    egui::{
        menu, Button, CentralPanel, CollapsingHeader, Context, Key, KeyboardShortcut, Modifiers,
        RichText, TextureOptions, TopBottomPanel, Window,
    },
    epaint::{Color32, ColorImage},
    App, CreationContext, Frame, Storage,
};
use egui::{Grid, Rgba, ScrollArea, SelectableLabel, SidePanel, TextureHandle, Vec2};
use egui_extras::RetainedImage;
use hashlink::LinkedHashSet;

use crate::{
    cpu::registers::Flag,
    ppu::{
        color_palette::{Chocolate, Green, Monochrome, ScreenColor},
        ppu::{LCD_HEIGHT, LCD_WIDTH},
    },
    ui::memory_viewer::MemoryViewer,
};
use crate::{emulator::Emulator, ppu::color_palette::COLOR_CORRECTION};

use super::{
    control_panel::ControlPanel,
    palette_picker::{Palette, PalettePicker},
    sound_settings::SoundSettings,
};
use crate::ui::frame_history::FrameHistory;

/// Shortcut for adding phosphor icons infront of text.
macro_rules! icon_text {
    ($icon:ident, $text:expr) => {
        format!("{} {}", egui_phosphor::regular::$icon, $text)
    };
}

/// Overarching struct that handles the emulator
/// and keeps track of UI elements since we are using
/// an immediate mode GUI.
pub struct Kevboy {
    emulator: Emulator,
    history: FrameHistory,

    texture: Option<TextureHandle>,
    frame_buffer: Vec<Color32>,

    mem_viewer: MemoryViewer,
    control_panel: ControlPanel,
    palette_picker: PalettePicker,
    sound_settings: SoundSettings,
    github_img: RetainedImage,

    recent_roms: LinkedHashSet<PathBuf>,
    is_vram_window_open: bool,

    playback_button_width: f32,
    pause: bool,
    right: bool,
    fast_forward: bool,

    integer_scaling: (bool, u8),
    blend: bool,
    color_correction: bool,
    channels: (Sender<FileType>, Receiver<FileType>),
}

enum FileType {
    Rom(Vec<u8>),
    SaveFile(Vec<u8>),
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

            texture: None,
            frame_buffer: [Green::WHITE].repeat(LCD_WIDTH * LCD_HEIGHT),

            mem_viewer: MemoryViewer::new(),
            control_panel: ControlPanel::new(cc),
            palette_picker: PalettePicker::new(cc),
            sound_settings: SoundSettings::new(cc),
            github_img: RetainedImage::from_svg_bytes(
                "gh",
                include_bytes!("../../icon/github-mark-white.svg"),
            )
            .unwrap(),

            recent_roms: eframe::get_value(cc.storage.unwrap(), "recent_roms").unwrap_or_default(),
            is_vram_window_open: false,

            playback_button_width: 0.0,
            pause: false,
            right: false,
            fast_forward: false,

            integer_scaling: (false, 0),
            blend: false,
            color_correction: false,
            channels: std::sync::mpsc::channel(),
        }
    }

    /// For starting the emulator from the command line
    pub fn with_rom(rom: &[u8], cc: &CreationContext) -> Self {
        let mut emulator = Emulator::new();
        emulator.load_rom(rom);

        let mut kevboy = Self::new(cc);
        kevboy.emulator = emulator;

        return kevboy;
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

        // #[cfg(target_arch = "wasm32")]
        // {
        //     // self.history.update(ctx, frame);
        //     // web_sys::window()
        //     //     .unwrap()
        //     //     .document()
        //     //     .unwrap()
        //     //     .set_title(&format!("Kevboy ({} fps)", self.history.fps()));
        // }

        #[cfg(target_arch = "wasm32")]
        loop {
            match self.channels.1.try_recv() {
                Ok(msg) => match msg {
                    FileType::Rom(rom) => {
                        self.emulator.load_rom(&rom);
                        self.mem_viewer = MemoryViewer::new_with_memory(&rom, true);
                    }
                    FileType::SaveFile(save) => {
                        self.emulator.load_rom(&self.emulator.rom.clone());
                        self.emulator.bus.cartridge.load_sram(&save);
                    }
                },
                Err(_) => break,
            }
        }

        // Load rom file when dropped on top of the GUI (wasm version)
        ctx.input(|c| {
            let dropped_files = &c.raw.dropped_files;
            let first_rom = dropped_files
                .iter()
                .find(|file| {
                    let extension = file.name.clone();
                    extension.ends_with(".gb") || extension.ends_with(".gbc")
                })
                .cloned();

            if let Some(file) = first_rom {
                let rom = file.bytes.unwrap();

                web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .set_title(&format!("Kevboy - {:#?}", file.name));

                self.emulator.load_rom(&rom);
                self.mem_viewer = MemoryViewer::new_with_memory(&rom, true);
            }
        });

        if let Some(tex) = &mut self.texture {
            tex.set(
                ColorImage {
                    size: [LCD_WIDTH, LCD_HEIGHT],
                    pixels: self.frame_buffer.clone(),
                },
                TextureOptions::NEAREST,
            );
        } else {
            self.texture = Some(ctx.load_texture(
                "fb",
                ColorImage::new([LCD_WIDTH, LCD_HEIGHT], Green::WHITE),
                TextureOptions::NEAREST,
            ));
        }

        // ----------------------------------
        //      Start of UI declarations
        // ----------------------------------

        TopBottomPanel::top("menu").show(ctx, |root| {
            menu::bar(root, |ui| {
                ui.menu_button("File", |ui| {
                    // Opens File Dialog filtered for the most common extensions.
                    // Inserts path into `recent_roms` and saves it into local storage.
                    // Then, loads the rom into the emulator and inits the memory viewer.
                    if ui.button(icon_text!(FILE_CODE, "Open ROM")).clicked() {
                        #[cfg(target_arch = "wasm32")]
                        {
                            let async_file = rfd::AsyncFileDialog::new()
                                .add_filter("Game Boy ROM", &["gb", "bin", "gbc"])
                                .pick_file();

                            let sender = self.channels.0.clone();

                            let file_future = async move {
                                let file = async_file.await;

                                if let Some(path) = file {
                                    web_sys::window()
                                        .unwrap()
                                        .document()
                                        .unwrap()
                                        .set_title(&format!("Kevboy - {:#?}", path.file_name()));

                                    let rom = path.read().await;
                                    sender.send(FileType::Rom(rom)).ok();
                                }
                            };

                            wasm_bindgen_futures::spawn_local(file_future);
                        }

                        ui.close_menu();
                    }

                    // Iterates through a copy of `recent_roms` and generates menu buttons from it
                    // so that all the most recently loaded roms are there. Only displays the file name, not the full path.
                    // Loads the emulator and memory viewer upon clicking a rom.
                    if ui.button(icon_text!(FILES, "Open Demo ROM")).clicked() {
                        let fairy_lake = include_bytes!("../../assets/fairylake.gb");
                        self.emulator.load_rom(fairy_lake);
                        self.mem_viewer = MemoryViewer::new_with_memory(fairy_lake, true);
                    }

                    ui.separator();

                    // Load save file and restarts the game,
                    // this is only enabled when a game is already loaded,
                    // so that we can determine what game the save file belongs to.
                    if ui
                        .add_enabled(
                            !self.emulator.rom.is_empty(),
                            Button::new(icon_text!(UPLOAD_SIMPLE, "Load Save")).shortcut_text(
                                ctx.format_shortcut(&KeyboardShortcut::new(
                                    Modifiers::CTRL,
                                    Key::L,
                                )),
                            ),
                        )
                        .clicked()
                    {
                        #[cfg(target_arch = "wasm32")]
                        {
                            let async_file = rfd::AsyncFileDialog::new()
                                .add_filter("Save file", &["sav"])
                                .pick_file();

                            let sender = self.channels.0.clone();

                            let file_future = async move {
                                let file = async_file.await;

                                if let Some(path) = file {
                                    let save_file = path.read().await;
                                    sender.send(FileType::SaveFile(save_file)).ok();
                                }
                            };

                            wasm_bindgen_futures::spawn_local(file_future);
                        }

                        #[cfg(not(target_arch = "wasm32"))] {
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
                    }

                    // Store save file, only enabled when game is already loaded.
                    if ui
                        .add_enabled(
                            !self.emulator.rom.is_empty(),
                            Button::new(icon_text!(DOWNLOAD_SIMPLE, "Store Save")).shortcut_text(
                                ctx.format_shortcut(&KeyboardShortcut::new(
                                    Modifiers::CTRL,
                                    Key::S,
                                )),
                            ),
                        )
                        .clicked()
                    {
                        #[cfg(target_arch = "wasm32")] {
                            let sram = self.emulator.bus.cartridge.dump_sram();

                            if let Some(sram) = sram {
                                let js_arr = js_sys::Uint8Array::new_with_length(sram.len() as u32);
                                js_arr.copy_from(&sram);

                                let byte_seq_arr = js_sys::Array::new_with_length(1);
                                byte_seq_arr.set(0, js_arr.into());

                                let file = web_sys::File::new_with_buffer_source_sequence_and_options(
                                    &byte_seq_arr.into(),
                                    &format!("{}.sav", self.emulator.bus.cartridge.title),
                                    web_sys::FilePropertyBag::new().type_("application/octet-stream"),
                                )
                                .unwrap();

                                let blob = web_sys::Blob::from(file);
                                let object_url = web_sys::Url::create_object_url_with_blob(&blob);

                                if let Ok(url) = object_url {
                                    web_sys::window().unwrap().open_with_url(&url).unwrap();
                                }
                            } else {
                                rfd::MessageDialog::new()
                                    .set_title("No saving was done!")
                                    .set_description("Nothing was saved as this cartridge does not support external RAM.")
                                    .show();
                            }
                        }

                        #[cfg(not(target_arch = "wasm32"))] {
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
                    }
                });

                // Options for changing controls and color palettes.
                // Both may open a new window and will save to local storage.
                ui.menu_button("Options", |ui| {
                    ui.menu_button(icon_text!(PALETTE, "Change palette"), |ui| {
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

                    ui.menu_button(icon_text!(MAGNIFYING_GLASS_PLUS, "Scaling"), |ui| {
                        let (force, scale) = &mut self.integer_scaling;

                        ui.checkbox(force, "Force integer scaling");
                        ui.separator();
                        ui.add_enabled_ui(*force, |ui| {
                            ui.radio_value(scale, 0, "Automatic");
                            ui.radio_value(scale, 1, "1x");
                            ui.radio_value(scale, 2, "2x");
                            ui.radio_value(scale, 3, "3x");
                            ui.radio_value(scale, 4, "4x");
                            ui.radio_value(scale, 5, "5x");
                        });
                    });

                    ui.separator();
                    ui.toggle_value(&mut self.blend, icon_text!(CARDS, "Frame blending")).on_hover_text("Slow on the web version!");
                    if ui.toggle_value(&mut self.color_correction, icon_text!(PAINT_BRUSH_HOUSEHOLD, "Color correction")).clicked() {
                        COLOR_CORRECTION.store(self.color_correction, Ordering::SeqCst);
                    }
                    ui.separator();

                    if ui.button(icon_text!(GAME_CONTROLLER, "Controls . . .")).clicked() {
                        self.control_panel.open = !self.control_panel.open;
                    };

                    if ui.button(icon_text!(SPEAKER_HIGH, "Sound . . .")).clicked() {
                        self.sound_settings.open = !self.sound_settings.open;
                    }
                });

                ui.menu_button("Debug", |ui| {
                    if ui.button(icon_text!(FLOPPY_DISK, "Show memory (hex)")).clicked() {
                        self.mem_viewer.open = !self.mem_viewer.open;
                    }
                    if ui.button(icon_text!(FRAME_CORNERS, "Open VRAM viewer")).clicked() {
                        self.is_vram_window_open = !self.is_vram_window_open;
                    }
                });
            });
        });

        // Optional information about register state, cartridge type and an "About" section
        // Can be expanded via the "R" button
        SidePanel::right("rp").resizable(false).show_animated(ctx, self.right, |ui| {
            ScrollArea::new([false, true]).show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(10.0);
                    ui.label(format!("FPS: {:.2}", self.history.fps()));
                    ui.add_space(10.0);

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

                    ui.add_space(10.0);

                    CollapsingHeader::new("About").default_open(true).show(ui, |ui| {
                        ui.label("This is a Color Game Boy emulator. \
                        It was made both as a learning exercise and because of the desire to create something emulation-related. \
                        It is not the most accurate emulator out there but it fares relatively well thanks to sub-instruction timing \
                        for example.");

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Supported MBCs:").strong());
                            ui.label(RichText::new("MBC0, MBC1, MBC2, MBC3, MBC5").monospace());
                        });

                        ui.horizontal(|ui| {
                            ui.image(self.github_img.texture_id(ctx), Vec2::splat(20.0));
                            ui.hyperlink("https://github.com/xkevio/kevboy");
                        });

                        ui.add_space(5.0);
                    });
                });
            });
        });

        // This panel holds both the game screen and the button group for resuming, pausing or stopping emulation
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.set_max_width(self.playback_button_width);
                    self.playback_button_width = ui
                        .group(|ui| {
                            ui.horizontal(|ui| {
                                if ui
                                    .add_sized([25.0, 25.0], Button::new(RichText::new("⏹").size(15.0)))
                                    .on_hover_text("Stop the emulation and reset the emulator state")
                                    .clicked()
                                {
                                    self.emulator.reset();
                                    self.frame_buffer.fill(Green::WHITE);
                                }

                                if ui
                                    .add_sized(
                                        [25.0, 25.0],
                                        Button::new(if !self.pause {
                                            RichText::new("⏸").size(15.0)
                                        } else {
                                            RichText::new("▶").size(15.0)
                                        }),
                                    )
                                    .on_hover_text("Pause / Resume the emulation")
                                    .clicked()
                                {
                                    self.pause = !self.pause;
                                }

                                if ui.add_sized([25.0, 25.0], SelectableLabel::new(self.fast_forward, RichText::new("⏩").size(15.0)))
                                    .on_hover_text("Fast forward (5x)\n(Disables sound on web!)")
                                    .clicked()
                                {
                                    self.fast_forward = !self.fast_forward;
                                    self.emulator.bus.apu.speed = self.fast_forward;
                                }

                            ui.separator();

                            if ui.add_sized([25.0, 25.0], SelectableLabel::new(self.right, RichText::new("R").size(15.0)))
                                .on_hover_text("Show right sidebar displaying extra information about registers,\nthe cartridge and an about section")
                                .clicked()
                            {
                                self.right = !self.right;
                            }
                            });
                        })
                        .response
                        .rect
                        .width();
                });

                ui.centered_and_justified(|ui| {
                    if let Some(tex) = &self.texture {
                        let raw_scale =
                            (ui.available_width().min(ui.available_height())) / LCD_WIDTH as f32;

                        let scale = match self.integer_scaling {
                            (false, _) => raw_scale,
                            (true, 0) => raw_scale.trunc(),
                            (true, sc) => sc as f32,
                        };

                        ui.image(tex.id(), tex.size_vec2() * scale);
                    }
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

        // Change and customize sound settings in this window
        if self.sound_settings.open {
            let mut sound_settings = self.sound_settings.open;
            Window::new("🔊 Volume")
                .open(&mut sound_settings)
                .resizable(false)
                .show(ctx, |ui| {
                    self.sound_settings
                        .show(ui, frame, &mut self.emulator.bus.apu);
                });
            self.sound_settings.open &= sound_settings;
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
                    self.emulator.bus.ppu.dump_bg_map(&self.emulator.bus.vram);

                    let pixels: Vec<Color32> = self
                        .emulator
                        .bus
                        .ppu
                        .raw_frame
                        .iter()
                        .map(|c| match *c {
                            ScreenColor::White(_) => self.palette_picker.colors["White"],
                            ScreenColor::LightGray(_) => self.palette_picker.colors["Light Gray"],
                            ScreenColor::Gray(_) => self.palette_picker.colors["Gray"],
                            ScreenColor::Black(_) => self.palette_picker.colors["Black"],
                            ScreenColor::FullColor(c, _) => c,
                        })
                        .collect();

                    let image = RetainedImage::from_color_image(
                        "vram",
                        ColorImage {
                            size: [256, 256],
                            pixels,
                        },
                    )
                    .with_options(TextureOptions::NEAREST);

                    image.show_size(ui, ui.available_size());
                });

            Window::new("💾 VRAM:0")
                .open(&mut self.is_vram_window_open)
                .show(ctx, |ui| {
                    let mut mem = MemoryViewer::new_with_memory(&self.emulator.bus.vram[0], false);
                    mem.show(ui);
                });

            Window::new("💾 VRAM:1")
                .open(&mut self.is_vram_window_open)
                .show(ctx, |ui| {
                    let mut mem = MemoryViewer::new_with_memory(&self.emulator.bus.vram[1], false);
                    mem.show(ui);
                });
        }

        // ----------------------------------
        //      End of UI declarations
        // ----------------------------------

        if !self.emulator.rom.is_empty() && !self.pause {
            self.run(ctx);
            ctx.request_repaint();
        }
    }
}

/// Second impl block for the run function
impl Kevboy {
    fn run(&mut self, ctx: &Context) {
        let double_factor = if self.emulator.bus.double_speed { 2 } else { 1 };

        // Poll keyboard and gamepad input once per frame.
        self.emulator.bus.joypad.tick(
            ctx,
            &mut self.emulator.bus.interrupt_handler,
            &self.control_panel.action_keys,
            &self.control_panel.direction_keys,
            &mut self.control_panel.gilrs,
        );

        while self.emulator.cycle_count < 17_556 * double_factor {
            for _ in 0..(4 * self.fast_forward as u8) {
                self.emulator.step();
            }
            self.emulator.cycle_count += self.emulator.step() as u16;
        }

        self.emulator
            .bus
            .apu
            .sink
            .set_volume(self.sound_settings.volume / 100.0);

        // Normal frame buffer for frontend, gets swapped for double buffering
        let frame_buffer = self
            .emulator
            .bus
            .ppu
            .ui_frame_buffer
            .iter()
            .map(|c| match *c {
                ScreenColor::White(_) => self.palette_picker.colors["White"],
                ScreenColor::LightGray(_) => self.palette_picker.colors["Light Gray"],
                ScreenColor::Gray(_) => self.palette_picker.colors["Gray"],
                ScreenColor::Black(_) => self.palette_picker.colors["Black"],
                ScreenColor::FullColor(c, _) => c,
            })
            .collect::<Vec<_>>();

        if self.blend {
            let old_frame = self.frame_buffer.clone();
            let new_frame = frame_buffer;
            self.frame_buffer = self.frame_blend(&old_frame, &new_frame);
        } else {
            self.frame_buffer = frame_buffer;
        }

        self.emulator.cycle_count = 0;
        self.emulator.bus.joypad.reset_pressed_keys();
    }

    // TODO: rewrite as shader, slow on web
    fn frame_blend(&self, old: &[Color32], new: &[Color32]) -> Vec<Color32> {
        new.iter()
            .zip(old)
            .map(|(n, o)| {
                let nc = Rgba::from_srgba_premultiplied(n.r(), n.g(), n.b(), n.a());
                let no = Rgba::from_srgba_premultiplied(o.r(), o.g(), o.b(), o.a());

                let c = nc + (no.multiply(0.5));
                let cc = c.to_srgba_unmultiplied();
                Color32::from_rgba_premultiplied(cc[0], cc[1], cc[2], cc[3])
            })
            .collect()
    }
}
