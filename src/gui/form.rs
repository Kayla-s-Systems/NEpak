#![forbid(unsafe_code)]

#[cfg(feature = "gui")]
use std::path::PathBuf;

#[cfg(feature = "gui")]
#[derive(Default, Clone)]
pub struct BuildForm {
    pub input_dir: String,
    pub output_pak: String,
    pub prefix: String,
    pub excludes_csv: String,
    pub compress: bool,
    pub zstd_level: i32,
}

#[cfg(feature = "gui")]
impl BuildForm {
    pub fn normalized_prefix(&self) -> String {
        let mut s = self.prefix.trim().replace('\\', "/");
        if s == "." {
            s.clear();
        }
        if !s.is_empty() && !s.ends_with('/') {
            s.push('/');
        }
        s
    }

    pub fn excludes(&self) -> Vec<String> {
        self.excludes_csv
            .split(',')
            .map(|x| x.trim())
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string())
            .collect()
    }

    pub fn ensure_pak_ext(p: &str) -> String {
        let p = p.trim();
        if p.is_empty() {
            return String::new();
        }
        let pb = PathBuf::from(p);
        if pb.extension().and_then(|e| e.to_str()).unwrap_or("") == "pak" {
            return pb.to_string_lossy().to_string();
        }
        let mut s = pb.to_string_lossy().to_string();
        if !s.ends_with('.') {
            s.push('.');
        }
        s.push_str("pak");
        s
    }

    pub fn to_args(&self) -> Result<(PathBuf, PathBuf, String, Vec<String>, bool, i32), String> {
        let input = PathBuf::from(self.input_dir.trim());
        if self.input_dir.trim().is_empty() {
            return Err("Input directory is empty".into());
        }
        if !input.exists() || !input.is_dir() {
            return Err("Input directory does not exist or is not a directory".into());
        }

        let out_s = Self::ensure_pak_ext(&self.output_pak);
        if out_s.is_empty() {
            return Err("Output .pak path is empty".into());
        }
        let output = PathBuf::from(out_s);

        let prefix = self.normalized_prefix();
        let excludes = self.excludes();

        let level = if self.compress {
            self.zstd_level.clamp(1, 22)
        } else {
            0
        };

        Ok((input, output, prefix, excludes, self.compress, level))
    }
}