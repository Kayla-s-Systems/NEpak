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
    pub fn ui_verify(&mut self, ui: &mut egui::Ui) {
        ui.label("Verify integrity: bounds + hashes.");
        ui.add_space(6.0);

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Pak");
                ui.text_edit_singleline(&mut self.verify_pak);
                if ui.button("Browse…").clicked() {
                    Self::browse_open_pak(&mut self.verify_pak);
                }
                if ui.add_enabled(!self.busy, egui::Button::new("Verify")).clicked() {
                    let pak_path = PathBuf::from(self.verify_pak.trim());
                    self.start_job(move |tx| {
                        let res = pak::verify(&pak_path).map_err(|e| e.to_string());
                        let _ = tx.send(JobMsg::VerifyDone(res));
                    });
                }
            });
        });
    }
}