#![forbid(unsafe_code)]

use blake3::Hasher;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::pak::build::build as build_impl;
use crate::pak::error::{PakError, PakResult};
use crate::pak::format::{EntryInfo, PayloadKind, MAGIC};
use crate::pak::io::{hex32, read_exact};
use crate::pak::read::read_index;

pub fn build(
    input: &Path,
    output: &Path,
    prefix: &str,
    excludes: &[String],
    compress: bool,
    zstd_level: i32,
) -> PakResult<()> {
    build_impl(input, output, prefix, excludes, compress, zstd_level)
}

/// Read pak index entries (without extracting payloads).
pub fn entries(pak: &Path) -> PakResult<Vec<EntryInfo>> {
    let mut f = File::open(pak)?;

    let head = read_exact::<8>(&mut f)?;
    if head != MAGIC {
        return Err(PakError::Invalid("bad header magic".into()));
    }

    let entries = read_index(&mut f)?;
    Ok(entries
        .into_iter()
        .map(|e| EntryInfo {
            path: e.path,
            payload_offset: e.payload_offset,
            payload_len: e.payload_len,
            raw_len: e.raw_len,
            payload_kind: match e.payload_kind {
                PayloadKind::Raw => "raw",
                PayloadKind::Zstd => "zstd",
            },
            raw_hash_hex: hex32(&e.raw_hash),
        })
        .collect())
}

pub fn list(pak: &Path, verbose: bool) -> PakResult<()> {
    let mut f = File::open(pak)?;

    let head = read_exact::<8>(&mut f)?;
    if head != MAGIC {
        return Err(PakError::Invalid("bad header magic".into()));
    }

    let entries = read_index(&mut f)?;
    for e in entries {
        if verbose {
            println!(
                "{}  off={} len={} raw={} kind={:?} hash={}",
                e.path,
                e.payload_offset,
                e.payload_len,
                e.raw_len,
                e.payload_kind,
                hex32(&e.raw_hash)
            );
        } else {
            println!("{}", e.path);
        }
    }
    Ok(())
}

pub fn extract(pak: &Path, output: &Path, filter: &[String]) -> PakResult<()> {
    let mut f = File::open(pak)?;

    let head = read_exact::<8>(&mut f)?;
    if head != MAGIC {
        return Err(PakError::Invalid("bad header magic".into()));
    }

    let entries = read_index(&mut f)?;
    std::fs::create_dir_all(output)?;

    for e in entries {
        if !filter.is_empty() && !filter.iter().any(|s| e.path.contains(s)) {
            continue;
        }

        f.seek(SeekFrom::Start(e.payload_offset))?;
        let mut payload = vec![0u8; e.payload_len as usize];
        f.read_exact(&mut payload)?;

        let raw = match e.payload_kind {
            PayloadKind::Raw => payload,
            PayloadKind::Zstd => {
                #[cfg(feature = "zstd")]
                {
                    zstd::decode_all(&payload[..])?
                }
                #[cfg(not(feature = "zstd"))]
                {
                    return Err(PakError::NoZstd);
                }
            }
        };

        let mut hasher = Hasher::new();
        hasher.update(&raw);
        let got: [u8; 32] = hasher.finalize().into();
        if got != e.raw_hash {
            return Err(PakError::Invalid(format!("hash mismatch for {}", e.path)));
        }

        let out_path = output.join(e.path.replace('/', &std::path::MAIN_SEPARATOR.to_string()));
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&out_path, &raw)?;
    }

    Ok(())
}

pub fn verify(pak: &Path) -> PakResult<()> {
    let mut f = File::open(pak)?;

    let head = read_exact::<8>(&mut f)?;
    if head != MAGIC {
        return Err(PakError::Invalid("bad header magic".into()));
    }

    let entries = read_index(&mut f)?;
    let file_len = f.metadata()?.len();

    for e in &entries {
        if e.payload_offset < 8 {
            return Err(PakError::Invalid(format!("payload offset under header: {}", e.path)));
        }
        if e.payload_offset + e.payload_len > file_len {
            return Err(PakError::Invalid(format!("payload outside file: {}", e.path)));
        }

        f.seek(SeekFrom::Start(e.payload_offset))?;
        let mut payload = vec![0u8; e.payload_len as usize];
        f.read_exact(&mut payload)?;

        let raw = match e.payload_kind {
            PayloadKind::Raw => payload,
            PayloadKind::Zstd => {
                #[cfg(feature = "zstd")]
                {
                    zstd::decode_all(&payload[..])?
                }
                #[cfg(not(feature = "zstd"))]
                {
                    return Err(PakError::NoZstd);
                }
            }
        };

        if raw.len() as u64 != e.raw_len {
            return Err(PakError::Invalid(format!("raw size mismatch: {}", e.path)));
        }

        let mut hasher = Hasher::new();
        hasher.update(&raw);
        let got: [u8; 32] = hasher.finalize().into();
        if got != e.raw_hash {
            return Err(PakError::Invalid(format!("hash mismatch: {}", e.path)));
        }
    }

    println!("ok: {} entries", entries.len());
    Ok(())
}