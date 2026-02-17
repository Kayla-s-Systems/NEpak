#![forbid(unsafe_code)]

use crate::pak;
use inquire::{Confirm, Text};
use std::path::{Path, PathBuf};

fn validate_dir(p: &str) -> Result<(), String> {
    let pb = PathBuf::from(p);
    if !pb.exists() {
        return Err("Path does not exist".to_string());
    }
    if !pb.is_dir() {
        return Err("Path is not a directory".to_string());
    }
    Ok(())
}

fn validate_output(p: &str) -> Result<(), String> {
    if p.trim().is_empty() {
        return Err("Output path is empty".to_string());
    }
    Ok(())
}

fn normalize_prefix(mut s: String) -> String {
    s = s.trim().replace('\\', "/");
    if s == "." {
        s.clear();
    }
    if !s.is_empty() && !s.ends_with('/') {
        s.push('/');
    }
    s
}

fn split_excludes(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.trim())
        .filter(|x| !x.is_empty())
        .map(|x| x.to_string())
        .collect()
}

fn ensure_pak_ext(p: &Path) -> PathBuf {
    if p.extension().and_then(|e| e.to_str()).unwrap_or("") == "pak" {
        return p.to_path_buf();
    }
    let mut s = p.to_string_lossy().to_string();
    if !s.ends_with('.') {
        s.push('.');
    }
    s.push_str("pak");
    PathBuf::from(s)
}

pub fn run() -> pak::PakResult<()> {
    println!("NEPAK Wizard\n");

    let input = Text::new("Input directory")
        .with_default("./assets")
        //.with_validator(validate_dir)
        .prompt()
        .map(PathBuf::from)
        .map_err(|e| pak::PakError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    let output_raw = Text::new("Output .pak file")
        .with_default("./assets.pak")
        //.with_validator(validate_output)
        .prompt()
        .map_err(|e| pak::PakError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    let output = ensure_pak_ext(Path::new(&output_raw));

    let prefix = Text::new("Mount prefix inside pak (optional)")
        .with_default("assets")
        .prompt()
        .map(normalize_prefix)
        .map_err(|e| pak::PakError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    let excludes_raw = Text::new("Excludes (comma-separated substrings, optional)")
        .with_default(".git,target")
        .prompt()
        .map_err(|e| pak::PakError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    let excludes = split_excludes(&excludes_raw);

    let compress = Confirm::new("Enable zstd compression?")
        .with_default(true)
        .prompt()
        .map_err(|e| pak::PakError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    let zstd_level = if compress {
        let lvl = Text::new("Zstd level (1..=22)")
            .with_default("6")
            .prompt()
            .map_err(|e| pak::PakError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        lvl.trim().parse::<i32>().unwrap_or(6).clamp(1, 22)
    } else {
        0
    };

    println!("\nBuild summary:");
    println!("  input   : {}", input.display());
    println!("  output  : {}", output.display());
    println!("  prefix  : {}", if prefix.is_empty() { "<none>" } else { &prefix });
    println!("  excludes: {}", if excludes.is_empty() { "<none>" } else { "(set)" });
    println!("  compress: {}", compress);
    if compress {
        println!("  zstd    : level {zstd_level}");
    }

    let proceed = Confirm::new("Proceed?").with_default(true).prompt().map_err(|e| {
        pak::PakError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
    })?;
    if !proceed {
        return Ok(());
    }

    pak::build(&input, &output, &prefix, &excludes, compress, zstd_level)
}
