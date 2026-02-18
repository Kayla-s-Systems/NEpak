#![forbid(unsafe_code)]

mod build;
mod error;
mod format;
mod io;
mod ops;
mod path;
mod read;

pub use build::{BuildProgress, BuildStage};

pub use error::{PakError, PakResult};
pub use format::{EntryInfo, FOOTER_MAGIC, MAGIC};

pub use ops::{build, build_with_progress, entries, extract, list, verify};