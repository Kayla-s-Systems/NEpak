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
    pub fn ui_extract(&mut self, ui: &mut egui::Ui) {
        ui.label("Extract a .pak into a directory.");
        ui.add_space(6.0);

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Pak");
                ui.text_edit_singleline(&mut self.extract_pak);
                if ui.button("Browse…").clicked() {
                    Self::browse_open_pak(&mut self.extract_pak);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Output dir");
                ui.text_edit_singleline(&mut self.extract_out);
                if ui.button("Browse…").clicked() {
                    Self::browse_folder(&mut self.extract_out);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Filter");
                ui.text_edit_singleline(&mut self.extract_filter_csv);
                ui.label("comma-separated substrings (optional)");
            });

            ui.add_space(8.0);

            if ui.add_enabled(!self.busy, egui::Button::new("Extract")).clicked() {
                let pak_path = PathBuf::from(self.extract_pak.trim());
                let out_dir = PathBuf::from(self.extract_out.trim());
                let filters: Vec<String> = self
                    .extract_filter_csv
                    .split(',')
                    .map(|x| x.trim())
                    .filter(|x| !x.is_empty())
                    .map(|x| x.to_string())
                    .collect();

                self.start_job(move |tx| {
                    let _ = tx.send(JobMsg::Log(format!(
                        "Extract: pak='{}' -> '{}'",
                        pak_path.display(),
                        out_dir.display()
                    )));
                    let res = pak::extract(&pak_path, &out_dir, &filters).map_err(|e| e.to_string());
                    let _ = tx.send(JobMsg::Done(res));
                });
            }
        });
    }
}