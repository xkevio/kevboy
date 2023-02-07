use eframe::{
    egui::{Grid, RichText, ScrollArea, Separator, TextStyle, Ui},
    epaint::Color32,
};

pub struct MemoryViewer {
    pub open: bool,
    memory: Vec<u8>,
    show_ascii: bool,
}

impl MemoryViewer {
    pub fn new() -> Self {
        Self {
            open: false,
            memory: Vec::new(),
            show_ascii: false,
        }
    }

    pub fn new_with_memory(memory: &[u8], show_ascii: bool) -> Self {
        Self {
            open: false,
            memory: memory.to_vec(),
            show_ascii,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let mem_chunks: Vec<_> = self.memory.chunks(16).collect();

        ui.checkbox(&mut self.show_ascii, "Show ASCII");
        ui.separator();

        ScrollArea::new([false, true]).show_rows(
            ui,
            ui.text_style_height(&TextStyle::Monospace),
            self.memory.len() / 16,
            |ui, range| {
                Grid::new("memory viewer").striped(true).show(ui, |ui| {
                    for line in range {
                        let chunk = mem_chunks[line];

                        ui.label(
                            RichText::new(format!("{:#06X}:", line * 16)).color(Color32::GOLD),
                        );

                        ui.label(
                            RichText::new(
                                chunk
                                    .iter()
                                    .map(|b| format!("{:02X?} ", b))
                                    .collect::<String>(),
                            )
                            .monospace(),
                        );

                        if self.show_ascii {
                            ui.horizontal(|ui| {
                                ui.add(Separator::default().vertical().spacing(3.0));

                                ui.label(
                                    RichText::new(
                                        chunk
                                            .iter()
                                            .map(|c| {
                                                if (32..=127).contains(c) {
                                                    *c as char
                                                } else {
                                                    '.'
                                                }
                                            })
                                            .collect::<String>(),
                                    )
                                    .monospace(),
                                );
                            });
                        }

                        ui.end_row();
                    }
                });
            },
        );
    }
}
