#![forbid(unsafe_code)]

#[cfg(feature = "gui")]
use eframe::egui;

#[cfg(feature = "gui")]
use crate::gui::app::NePakApp;

#[cfg(feature = "gui")]
impl NePakApp {
    pub fn ui_logs(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.label("Log");
        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
            for l in &self.logs {
                ui.monospace(l);
            }
        });
    }
}