#![forbid(unsafe_code)]

#[cfg(feature = "gui")]
use eframe::egui;

#[cfg(feature = "gui")]
use std::path::PathBuf;

#[cfg(feature = "gui")]
use crate::gui::{app::NePakApp, jobs::JobMsg};

#[cfg(feature = "gui")]
use crate::pak;

#[cfg(feature = "gui")]
impl NePakApp {
    pub fn ui_list(&mut self, ui: &mut egui::Ui) {
        ui.label("Inspect a .pak index (paths, sizes, compression).");
        ui.add_space(6.0);

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Pak");
                ui.text_edit_singleline(&mut self.list_pak);
                if ui.button("Browse…").clicked() {
                    Self::browse_open_pak(&mut self.list_pak);
                }
                if ui.add_enabled(!self.busy, egui::Button::new("Load")).clicked() {
                    let pak_path = PathBuf::from(self.list_pak.trim());
                    self.start_job(move |tx| {
                        let res = pak::entries(&pak_path).map_err(|e| e.to_string());
                        let _ = tx.send(JobMsg::ListDone(res));
                    });
                }
            });

            if let Some(e) = &self.entries_err {
                ui.colored_label(egui::Color32::LIGHT_RED, e);
            }

            ui.add_space(6.0);

            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.monospace(format!("entries: {}", self.entries.len()));
                });
                ui.separator();

                for e in &self.entries {
                    ui.horizontal(|ui| {
                        ui.monospace(&e.path);
                        ui.add_space(8.0);
                        ui.label(format!(
                            "raw={} payload={} kind={}",
                            e.raw_len,
                            e.payload_len,
                            e.payload_kind
                        ));
                    });
                }
            });
        });
    }
}