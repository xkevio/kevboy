use eframe::egui::{Button, Context, Frame, Grid, Separator, TextEdit, Ui, Window};

pub struct ControlPanel {
    pub open: bool,
    direction_keys: [(String, String); 4],
    action_keys: [(String, String); 4],
}

impl Default for ControlPanel {
    fn default() -> Self {
        Self {
            open: Default::default(),
            direction_keys: [
                ("Up".to_string(), "W".to_string()),
                ("Down".to_string(), "S".to_string()),
                ("Left".to_string(), "A".to_string()),
                ("Right".to_string(), "D".to_string()),
            ],
            action_keys: [
                ("Start".to_string(), "Enter".to_string()),
                ("Select".to_string(), "Q".to_string()),
                ("A".to_string(), "P".to_string()),
                ("B".to_string(), "O".to_string()),
            ],
        }
    }
}

impl ControlPanel {
    pub fn show(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add_space(ui.available_width() / 7.0);

            Frame::none().show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Direction");
                    Grid::new("direction").num_columns(2).show(ui, |ui| {
                        for (k, v) in &mut self.direction_keys {
                            ui.label(format!("{k}: "));
                            let response = ui.add(
                                TextEdit::singleline(&mut v.clone())
                                    .desired_width(50.0)
                                    .hint_text(v.clone()),
                            );

                            if response.has_focus() {
                                let buttons = &ctx.input().keys_down;
                                if !buttons.is_empty() {
                                    *v = format!("{:?}", buttons.iter().nth(0).unwrap());
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
                        for (k, v) in &mut self.action_keys {
                            ui.label(format!("{k}: "));
                            let response = ui.add(
                                TextEdit::singleline(&mut v.clone())
                                    .desired_width(50.0)
                                    .hint_text(v.clone()),
                            );

                            if response.has_focus() {
                                let buttons = &ctx.input().keys_down;
                                if !buttons.is_empty() {
                                    *v = format!("{:?}", buttons.iter().nth(0).unwrap());
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
        ui.vertical_centered(|ui| {
            ui.add(Button::new("Apply"));
        });
    }
}
