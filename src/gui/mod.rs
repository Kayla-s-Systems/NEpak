#![forbid(unsafe_code)]

#[cfg(feature = "gui")]
mod app;
#[cfg(feature = "gui")]
mod form;
#[cfg(feature = "gui")]
mod jobs;
#[cfg(feature = "gui")]
mod tabs;
#[cfg(feature = "gui")]
mod ui;

#[cfg(feature = "gui")]
use crate::pak;

#[cfg(feature = "gui")]
pub fn run() -> pak::PakResult<()> {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "NEPAK",
        native_options,
        Box::new(|_cc| Box::new(app::NePakApp::default())),
    )
        .map_err(|e| {
            pak::PakError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("eframe: {e}"),
            ))
        })
}

#[cfg(not(feature = "gui"))]
pub fn run() -> crate::pak::PakResult<()> {
    Err(crate::pak::PakError::Invalid(
        "nepak was built without feature 'gui'".into(),
    ))
}