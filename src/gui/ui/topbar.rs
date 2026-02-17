#![forbid(unsafe_code)]

#[cfg(feature = "gui")]
use eframe::egui;

#[cfg(feature = "gui")]
use crate::gui::{app::NePakApp, tabs::Tab};

#[cfg(feature = "gui")]
impl NePakApp {
    pub fn ui_topbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("NEPAK");
            ui.separator();
            ui.label("NewEngine PakBuilder");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Clear log").clicked() {
                    self.logs.clear();
                }
                ui.separator();
                ui.label(&self.status);
            });
        });
        ui.separator();

        ui.horizontal(|ui| {
            let build = ui.selectable_label(matches!(self.tab, Tab::Build), "Build");
            let list = ui.selectable_label(matches!(self.tab, Tab::List), "List");
            let extract = ui.selectable_label(matches!(self.tab, Tab::Extract), "Extract");
            let verify = ui.selectable_label(matches!(self.tab, Tab::Verify), "Verify");

            if build.clicked() {
                self.tab = Tab::Build;
            }
            if list.clicked() {
                self.tab = Tab::List;
            }
            if extract.clicked() {
                self.tab = Tab::Extract;
            }
            if verify.clicked() {
                self.tab = Tab::Verify;
            }

            ui.add_space(12.0);

            if self.busy {
                ui.label("Running…");
                ui.add(egui::Spinner::new());
            }
        });
        ui.separator();
    }
}