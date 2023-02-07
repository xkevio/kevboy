use std::collections::BTreeMap;

use eframe::egui::{Button, Grid, Ui};

pub struct PalettePicker {
    pub open: bool,
    pub colors: BTreeMap<String, [f32; 3]>,
}

impl Default for PalettePicker {
    fn default() -> Self {
        Self {
            open: Default::default(),
            colors: BTreeMap::from([
                ("Black".into(), [0.0, 0.0, 0.0]),
                ("Gray".into(), [0.0, 0.0, 0.0]),
                ("Light Gray".into(), [0.0, 0.0, 0.0]),
                ("White".into(), [0.0, 0.0, 0.0]),
            ]),
        }
    }
}

impl PalettePicker {
    pub fn show(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Choose custom colors");
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 3.0);
                Grid::new("colors").show(ui, |ui| {
                    for (k, v) in &mut self.colors {
                        ui.label(format!("{k}: "));
                        ui.color_edit_button_rgb(v);
                        ui.end_row();
                    }
                });
                ui.add_space(ui.available_width() / 3.0);
            });

            ui.add_space(5.0);
            ui.add(Button::new("Apply"));
        });
    }
}
