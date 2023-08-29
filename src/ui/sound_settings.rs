use eframe::CreationContext;
use egui::{RichText, Slider, Ui};

use crate::apu::APU;

pub struct SoundSettings {
    pub open: bool,
    pub volume: f32,
}

impl SoundSettings {
    pub fn new(cc: &CreationContext) -> Self {
        let volume = if let Some(storage) = cc.storage {
            if let Some(saved_volume) = eframe::get_value::<f32>(storage, "volume") {
                saved_volume
            } else {
                50.0
            }
        } else {
            50.0
        };

        Self {
            open: false,
            volume,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, frame: &mut eframe::Frame, apu: &mut APU) {
        ui.vertical_centered(|ui| {
            ui.heading("Change sound settings");
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 4.0);
                ui.label("Volume:");
                ui.add(Slider::new(&mut self.volume, 0.0..=100.0));
                ui.add_space(ui.available_width() / 4.0);
            });

            ui.add_space(5.0);
            ui.separator();

            ui.label(RichText::new("Enable separate channels:").size(15.0));
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 4.0);
                ui.checkbox(&mut apu.ch_enable.0, "Channel 1");
                ui.checkbox(&mut apu.ch_enable.1, "Channel 2");
                ui.add_space(ui.available_width() / 4.0);
            });
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 4.0);
                ui.checkbox(&mut apu.ch_enable.2, "Channel 3");
                ui.checkbox(&mut apu.ch_enable.3, "Channel 4");
                ui.add_space(ui.available_width() / 4.0);
            });

            ui.add_space(5.0);
            ui.separator();

            if ui
                .button("Apply")
                .on_hover_text("Saves the volume to a file")
                .clicked()
            {
                if let Some(storage) = frame.storage_mut() {
                    eframe::set_value(storage, "volume", &self.volume);
                    storage.flush();

                    self.open = false;
                }
            }
        });
    }
}
