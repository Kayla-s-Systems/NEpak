#![forbid(unsafe_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PakError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid pak: {0}")]
    Invalid(String),

    #[error("path is outside input dir: {0}")]
    Outside(String),

    #[error("compression requested but nepak was built without zstd feature")]
    NoZstd,
}

pub type PakResult<T> = Result<T, PakError>;