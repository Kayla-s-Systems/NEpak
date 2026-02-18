#![forbid(unsafe_code)]

#[cfg(feature = "gui")]
use eframe::egui;

#[cfg(feature = "gui")]
use crate::gui::{app::NePakApp, jobs::JobMsg};

#[cfg(feature = "gui")]
use crate::pak;

#[cfg(feature = "gui")]
impl NePakApp {
    pub fn ui_build(&mut self, ui: &mut egui::Ui) {
        ui.label("Create a deterministic .pak from an input directory.");
        ui.add_space(6.0);

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Input dir");
                ui.text_edit_singleline(&mut self.build.input_dir);
                if ui.button("Browse…").clicked() {
                    Self::browse_folder(&mut self.build.input_dir);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Output .pak");
                ui.text_edit_singleline(&mut self.build.output_pak);
                if ui.button("Browse…").clicked() {
                    Self::browse_save_pak(&mut self.build.output_pak);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Prefix");
                ui.text_edit_singleline(&mut self.build.prefix);
                ui.label("→ mounted as '<prefix>/path'");
            });

            ui.horizontal(|ui| {
                ui.label("Excludes");
                ui.text_edit_singleline(&mut self.build.excludes_csv);
                ui.label("comma-separated substrings");
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.build.compress, "zstd compression");
                ui.add_enabled(
                    self.build.compress,
                    egui::Slider::new(&mut self.build.zstd_level, 1..=22).text("level"),
                );
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let can_run = !self.busy;
                if ui.add_enabled(can_run, egui::Button::new("Build")).clicked() {
                    let args = self.build.to_args();
                    match args {
                        Ok((input, output, prefix, excludes, compress, level)) => {
                            self.push_log(format!(
                                "Build: input='{}' output='{}' prefix='{}' compress={} level={}",
                                input.display(),
                                output.display(),
                                if prefix.is_empty() { "<none>" } else { &prefix },
                                compress,
                                level
                            ));

                            self.start_job(move |tx| {
                                let _ = tx.send(JobMsg::Log("Scanning + building…".into()));
                                let res = pak::build_with_progress(
                                    &input,
                                    &output,
                                    &prefix,
                                    &excludes,
                                    compress,
                                    level,
                                    |p| {
                                        let _ = tx.send(JobMsg::Progress {
                                            stage: p.stage.as_str().to_string(),
                                            done: p.done,
                                            total: p.total,
                                            item: p.current,
                                        });
                                    },
                                )
                                .map_err(|e| e.to_string());
                                let _ = tx.send(JobMsg::Done(res));
                            });
                        }
                        Err(e) => {
                            self.status = format!("Error: {e}");
                            self.push_log(self.status.clone());
                        }
                    }
                }
            });

            if self.busy {
                ui.add_space(10.0);
                let total = self.progress_total.max(1);
                let frac = (self.progress_done.min(total) as f32) / (total as f32);

                let mut label = if self.progress_stage.is_empty() {
                    "Working…".to_string()
                } else {
                    self.progress_stage.clone()
                };
                if !self.progress_item.is_empty() {
                    // Keep the UI stable and readable.
                    let item = if self.progress_item.len() > 64 {
                        format!("{}…", &self.progress_item[..64])
                    } else {
                        self.progress_item.clone()
                    };
                    label.push_str(&format!("  |  {item}"));
                }

                ui.add(egui::ProgressBar::new(frac).show_percentage().text(label));
            }
        });
    }
}