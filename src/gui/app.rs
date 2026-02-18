#![forbid(unsafe_code)]

#[cfg(feature = "gui")]
use eframe::egui;

#[cfg(feature = "gui")]
use std::sync::mpsc;

#[cfg(feature = "gui")]
use crate::gui::form::BuildForm;

#[cfg(feature = "gui")]
use crate::gui::jobs::JobMsg;

#[cfg(feature = "gui")]
use crate::gui::tabs::Tab;

#[cfg(feature = "gui")]
pub struct NePakApp {
    pub tab: Tab,

    pub build: BuildForm,
    pub list_pak: String,
    pub extract_pak: String,
    pub extract_out: String,
    pub extract_filter_csv: String,
    pub verify_pak: String,

    pub entries: Vec<crate::pak::EntryInfo>,
    pub entries_err: Option<String>,

    pub logs: Vec<String>,
    pub status: String,
    pub busy: bool,

    pub progress_stage: String,
    pub progress_done: u64,
    pub progress_total: u64,
    pub progress_item: String,

    pub rx: Option<mpsc::Receiver<JobMsg>>,
}

#[cfg(feature = "gui")]
impl Default for NePakApp {
    fn default() -> Self {
        Self {
            tab: Tab::Build,
            build: BuildForm {
                input_dir: "./assets".into(),
                output_pak: "./assets.pak".into(),
                prefix: "assets".into(),
                excludes_csv: ".git,target".into(),
                compress: true,
                zstd_level: 6,
            },
            list_pak: "./assets.pak".into(),
            extract_pak: "./assets.pak".into(),
            extract_out: "./assets_extracted".into(),
            extract_filter_csv: "".into(),
            verify_pak: "./assets.pak".into(),
            entries: Vec::new(),
            entries_err: None,
            logs: vec!["NEPAK GUI ready.".into()],
            status: String::new(),
            busy: false,
            progress_stage: String::new(),
            progress_done: 0,
            progress_total: 0,
            progress_item: String::new(),
            rx: None,
        }
    }
}

#[cfg(feature = "gui")]
impl NePakApp {
    pub fn push_log(&mut self, s: impl Into<String>) {
        let s = s.into();
        self.logs.push(s);
        if self.logs.len() > 5000 {
            let drain = self.logs.len().saturating_sub(5000);
            self.logs.drain(0..drain);
        }
    }

    pub fn poll_jobs(&mut self) {
        let mut rx = match self.rx.take() {
            Some(rx) => rx,
            None => return,
        };

        let mut done = false;

        while let Ok(msg) = rx.try_recv() {
            match msg {
                JobMsg::Log(s) => self.push_log(s),

                JobMsg::Progress {
                    stage,
                    done,
                    total,
                    item,
                } => {
                    self.progress_stage = stage;
                    self.progress_done = done;
                    self.progress_total = total;
                    self.progress_item = item.unwrap_or_default();
                }

                JobMsg::Done(r) => {
                    self.busy = false;
                    self.progress_stage.clear();
                    self.progress_done = 0;
                    self.progress_total = 0;
                    self.progress_item.clear();
                    self.status = match r {
                        Ok(()) => "Done.".into(),
                        Err(e) => format!("Error: {e}"),
                    };
                    self.push_log(self.status.clone());
                    done = true;
                    break;
                }

                JobMsg::ListDone(r) => {
                    self.busy = false;
                    self.progress_stage.clear();
                    self.progress_done = 0;
                    self.progress_total = 0;
                    self.progress_item.clear();
                    match r {
                        Ok(list) => {
                            self.entries = list;
                            self.entries_err = None;
                            self.status = format!("Loaded {} entries.", self.entries.len());
                        }
                        Err(e) => {
                            self.entries.clear();
                            self.entries_err = Some(e.clone());
                            self.status = format!("Error: {e}");
                        }
                    }
                    self.push_log(self.status.clone());
                    done = true;
                    break;
                }

                JobMsg::VerifyDone(r) => {
                    self.busy = false;
                    self.progress_stage.clear();
                    self.progress_done = 0;
                    self.progress_total = 0;
                    self.progress_item.clear();
                    self.status = match r {
                        Ok(()) => "Pak verified OK.".into(),
                        Err(e) => format!("Error: {e}"),
                    };
                    self.push_log(self.status.clone());
                    done = true;
                    break;
                }
            }
        }

        if !done {
            self.rx = Some(rx);
        } else {
            self.rx = None;
        }
    }

    pub fn start_job(&mut self, f: impl FnOnce(mpsc::Sender<JobMsg>) + Send + 'static) {
        if self.busy {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        self.busy = true;
        self.status = "Working...".into();
        self.progress_stage = "Starting".into();
        self.progress_done = 0;
        self.progress_total = 0;
        self.progress_item.clear();
        std::thread::spawn(move || f(tx));
    }

    pub fn browse_folder(target: &mut String) {
        if let Some(p) = rfd::FileDialog::new().pick_folder() {
            *target = p.to_string_lossy().to_string();
        }
    }

    pub fn browse_save_pak(target: &mut String) {
        if let Some(p) = rfd::FileDialog::new()
            .add_filter("NEPAK", &["pak"])
            .save_file()
        {
            *target = p.to_string_lossy().to_string();
        }
    }

    pub fn browse_open_pak(target: &mut String) {
        if let Some(p) = rfd::FileDialog::new()
            .add_filter("NEPAK", &["pak"])
            .pick_file()
        {
            *target = p.to_string_lossy().to_string();
        }
    }
}

#[cfg(feature = "gui")]
impl eframe::App for NePakApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_jobs();

        egui::TopBottomPanel::top("top").show(ctx, |ui| self.ui_topbar(ui));

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.tab {
                Tab::Build => self.ui_build(ui),
                Tab::List => self.ui_list(ui),
                Tab::Extract => self.ui_extract(ui),
                Tab::Verify => self.ui_verify(ui),
            }
            self.ui_logs(ui);
        });
    }
}

#[cfg(feature = "gui")]
use crate::gui::ui::{build::*, extract::*, list::*, logs::*, topbar::*, verify::*};
