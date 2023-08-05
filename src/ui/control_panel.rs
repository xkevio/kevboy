use eframe::{
    egui::{Context, Grid, Key, Separator, TextEdit, Ui},
    CreationContext,
};
use egui::{Button, Frame};
use gilrs::{Button as CButton, Gilrs};
use hashlink::LinkedHashMap;

pub struct ControlPanel {
    pub open: bool,

    // LinkedHashMap keeps insertion order, which we want
    // since we generate UI elements from this map.
    pub direction_keys: LinkedHashMap<String, (Key, CButton)>,
    pub action_keys: LinkedHashMap<String, (Key, CButton)>,

    pub gilrs: Gilrs,

    button_width: f32,
}

impl Default for ControlPanel {
    fn default() -> Self {
        Self {
            open: Default::default(),
            direction_keys: LinkedHashMap::from_iter([
                ("Right".into(), (Key::D, CButton::DPadRight)),
                ("Left".into(), (Key::A, CButton::DPadLeft)),
                ("Up".into(), (Key::W, CButton::DPadUp)),
                ("Down".into(), (Key::S, CButton::DPadDown)),
            ]),
            action_keys: LinkedHashMap::from_iter([
                ("A".into(), (Key::P, CButton::South)),
                ("B".into(), (Key::O, CButton::East)),
                ("Select".into(), (Key::Q, CButton::Select)),
                ("Start".into(), (Key::Enter, CButton::Start)),
            ]),
            gilrs: Gilrs::new().unwrap(),
            button_width: 0.0,
        }
    }
}

impl ControlPanel {
    pub fn new(cc: &CreationContext) -> Self {
        if let Some(storage) = cc.storage {
            if let (Some(dir_controls), Some(action_controls)) = (
                eframe::get_value::<LinkedHashMap<String, (Key, CButton)>>(storage, "dir_controls"),
                eframe::get_value::<LinkedHashMap<String, (Key, CButton)>>(
                    storage,
                    "action_controls",
                ),
            ) {
                Self {
                    open: Default::default(),
                    direction_keys: dir_controls,
                    action_keys: action_controls,
                    gilrs: Gilrs::new().unwrap(),
                    button_width: 0.0,
                }
            } else {
                ControlPanel::default()
            }
        } else {
            ControlPanel::default()
        }
    }

    pub fn show(&mut self, ctx: &Context, ui: &mut Ui, frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            ui.add_space(ui.available_width() / 7.0);

            Frame::none().show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Direction");
                    Grid::new("direction").num_columns(2).show(ui, |ui| {
                        for (name, (key, _)) in &mut self.direction_keys {
                            ui.label(format!("{name}: "));
                            let response = ui.add(
                                TextEdit::singleline(&mut (*key).name().to_string())
                                    .desired_width(50.0)
                                    .lock_focus(true)
                                    .hint_text((*key).name()),
                            );

                            if response.has_focus() || response.lost_focus() {
                                let buttons = ctx.input(|i| i.keys_down.clone());
                                if !buttons.is_empty() {
                                    *key = *buttons.iter().next().unwrap();
                                }
                            }

                            ui.end_row();
                        }
                    });
                });
            });

            ui.add(Separator::default().vertical());

            Frame::none().show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Action");
                    Grid::new("action").num_columns(2).show(ui, |ui| {
                        for (name, (key, _)) in &mut self.action_keys {
                            ui.label(format!("{name}: "));
                            let response = ui.add(
                                TextEdit::singleline(&mut (*key).name().to_string())
                                    .desired_width(50.0)
                                    .lock_focus(true)
                                    .hint_text((*key).name()),
                            );

                            if response.has_focus() || response.lost_focus() {
                                let buttons = ctx.input(|i| i.keys_down.clone());
                                if !buttons.is_empty() {
                                    *key = *buttons.iter().next().unwrap();
                                }
                            }

                            ui.end_row();
                        }
                    });
                });
            });

            ui.add_space(ui.available_width() / 7.0);
        });

        ui.add_space(5.0);
        ui.separator();
        ui.vertical_centered(|ui| {
            ui.set_max_width(self.button_width);

            self.button_width = ui
                .horizontal(|ui| {
                    if ui
                        .button("Apply")
                        .on_hover_text("Apply new controls and save them to a file")
                        .clicked()
                    {
                        if let Some(storage) = frame.storage_mut() {
                            eframe::set_value(storage, "dir_controls", &self.direction_keys);
                            eframe::set_value(storage, "action_controls", &self.action_keys);
                            storage.flush();
                        }

                        self.open = false;
                    }
                    if ui
                        .add_enabled(false, Button::new("Reset"))
                        .on_disabled_hover_text("Reset changes to controls")
                        .clicked()
                    {
                        // TODO
                    }
                })
                .response
                .rect
                .width();
        });
    }
}
