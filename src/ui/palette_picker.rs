use crate::ppu::color_palette::{Chocolate, Green, Monochrome};
use eframe::{
    egui::{Grid, Ui},
    epaint::Color32,
    CreationContext,
};
use hashlink::LinkedHashMap;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Palette {
    Monochrome(Monochrome),
    Green(Green),
    Chocolate(Chocolate),
    Custom,
}

/// UI for customizing the color palette
pub struct PalettePicker {
    pub open: bool,
    pub current_palette: Palette,

    pub colors: LinkedHashMap<String, Color32>,
    prev_colors: LinkedHashMap<String, Color32>,

    button_width: f32,
}

impl Default for PalettePicker {
    fn default() -> Self {
        Self {
            open: Default::default(),
            current_palette: Palette::Green(Green),
            colors: LinkedHashMap::from_iter([
                ("Black".into(), Green::BLACK),
                ("Gray".into(), Green::GRAY),
                ("Light Gray".into(), Green::LIGHT_GRAY),
                ("White".into(), Green::WHITE),
            ]),
            prev_colors: LinkedHashMap::from_iter([
                ("Black".into(), Green::BLACK),
                ("Gray".into(), Green::GRAY),
                ("Light Gray".into(), Green::LIGHT_GRAY),
                ("White".into(), Green::WHITE),
            ]),
            button_width: 0.0,
        }
    }
}

impl PalettePicker {
    pub fn new(cc: &CreationContext) -> Self {
        if let Some(storage) = cc.storage {
            if let Some(colors) = eframe::get_value::<LinkedHashMap<_, _>>(storage, "colors") {
                let prev_colors = colors.clone();
                Self {
                    open: Default::default(),
                    current_palette: Palette::Custom,
                    colors,
                    prev_colors,
                    button_width: 0.0,
                }
            } else {
                PalettePicker::default()
            }
        } else {
            PalettePicker::default()
        }
    }

    pub fn change_colors(
        &mut self,
        black: &Color32,
        gray: &Color32,
        light_gray: &Color32,
        white: &Color32,
    ) {
        self.colors["Black"] = *black;
        self.colors["Gray"] = *gray;
        self.colors["Light Gray"] = *light_gray;
        self.colors["White"] = *white;
    }

    pub fn show(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        ui.vertical_centered(|ui| {
            ui.heading("Choose custom colors");
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 3.0);
                Grid::new("colors").show(ui, |ui| {
                    for (k, v) in &mut self.colors {
                        ui.label(format!("{k}: "));
                        ui.color_edit_button_srgba(v);
                        ui.end_row();
                    }
                });
                ui.add_space(ui.available_width() / 3.0);
            });

            ui.add_space(5.0);
            ui.separator();
            ui.vertical_centered(|ui| {
                ui.set_max_width(self.button_width);
                self.button_width = ui
                    .horizontal(|ui| {
                        // "Apply" saves the chosen colors to an external file (egui storage)
                        if ui
                            .button("Apply")
                            .on_hover_text("Saves the palette to a file")
                            .clicked()
                        {
                            if let Some(storage) = frame.storage_mut() {
                                eframe::set_value(storage, "colors", &self.colors);
                                storage.flush();
                            }

                            self.prev_colors = self.colors.clone();
                            self.open = false;
                        }

                        // "Reset" resets the color changes to whichever palette was active before "Apply" was clicked
                        if ui
                            .button("Reset")
                            .on_hover_text("Reset to currently saved palette")
                            .clicked()
                        {
                            self.colors = self.prev_colors.clone();
                        }
                    })
                    .response
                    .rect
                    .width();
            });
        });
    }
}
