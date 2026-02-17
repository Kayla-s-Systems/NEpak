#![forbid(unsafe_code)]

use std::path::Path;

use crate::pak::error::{PakError, PakResult};

pub fn normalize_rel_path(input_root: &Path, file_path: &Path) -> PakResult<String> {
    let rel = file_path
        .strip_prefix(input_root)
        .map_err(|_| PakError::Outside(file_path.to_string_lossy().into_owned()))?;

    let mut out = String::new();
    for (i, comp) in rel.components().enumerate() {
        if i != 0 {
            out.push('/');
        }
        out.push_str(&comp.as_os_str().to_string_lossy());
    }

    while out.starts_with('/') {
        out.remove(0);
    }
    out = out.replace('\\', "/");

    if out.is_empty() {
        return Err(PakError::Invalid("empty relative path".into()));
    }

    Ok(out)
}

pub fn prefixed(prefix: &str, rel: &str) -> String {
    if prefix.is_empty() {
        return rel.to_string();
    }
    let mut p = prefix.replace('\\', "/");
    if !p.ends_with('/') {
        p.push('/');
    }
    let r = rel.trim_start_matches('/');
    format!("{p}{r}")
}

pub fn should_exclude(norm_path: &str, excludes: &[String]) -> bool {
    excludes.iter().any(|e| !e.is_empty() && norm_path.contains(e))
}