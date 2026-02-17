#![forbid(unsafe_code)]

use blake3::Hasher;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::pak::error::{PakError, PakResult};
use crate::pak::format::{Entry, PayloadKind, FOOTER_MAGIC, MAGIC};
use crate::pak::io::{write_u32, write_u64};
use crate::pak::path::{normalize_rel_path, prefixed, should_exclude};

/// NEPAK v1 layout:
/// - [MAGIC 8]
/// - payload blobs (raw or compressed)
/// - index:
///   - [MAGIC 8]
///   - [u32 entry_count]
///   - entries...
///     - [u16 path_len][path bytes UTF-8]
///     - [u64 payload_offset]
///     - [u64 payload_len]
///     - [u64 raw_len]
///     - [u8 payload_kind]
///     - [u8 raw_hash[32]]
/// - footer:
///   - [FOOTER_MAGIC 8]
///   - [u64 index_offset]
///   - [u64 index_len]
///   - [u32 index_hash (blake3 truncated to u32)]
///   - [u32 reserved]
///
/// Determinism rules:
/// - paths are normalized to forward slashes
/// - entries are sorted lexicographically by path bytes
pub fn build(
    input: &Path,
    output: &Path,
    prefix: &str,
    excludes: &[String],
    compress: bool,
    zstd_level: i32,
) -> PakResult<()> {
    if compress {
        #[cfg(not(feature = "zstd"))]
        {
            let _ = zstd_level;
            return Err(PakError::NoZstd);
        }
    }

    let mut files: Vec<(String, PathBuf)> = Vec::new();
    for ent in WalkDir::new(input).follow_links(false).into_iter() {
        let ent = ent.map_err(|e| {
            let msg = e.to_string();
            let io = e
                .into_io_error()
                .unwrap_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, msg));
            PakError::Io(io)
        })?;

        if !ent.file_type().is_file() {
            continue;
        }

        let rel = normalize_rel_path(input, ent.path())?;
        let logical = prefixed(prefix, &rel);
        if should_exclude(&logical, excludes) {
            continue;
        }
        files.push((logical, ent.path().to_path_buf()));
    }

    files.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

    let mut out = File::create(output)?;
    out.write_all(&MAGIC)?;

    let mut entries: Vec<Entry> = Vec::with_capacity(files.len());

    for (logical, physical) in files {
        let payload_offset = out.stream_position()?;

        let mut f = File::open(&physical)?;
        let mut raw = Vec::new();
        f.read_to_end(&mut raw)?;

        let raw_len: u64 = raw.len() as u64;
        let mut hasher = Hasher::new();
        hasher.update(&raw);
        let raw_hash: [u8; 32] = hasher.finalize().into();

        let (payload, kind) = if compress {
            #[cfg(feature = "zstd")]
            {
                let mut encoder = zstd::Encoder::new(Vec::new(), zstd_level)?;
                encoder.write_all(&raw)?;
                let payload = encoder.finish()?;
                (payload, PayloadKind::Zstd)
            }
            #[cfg(not(feature = "zstd"))]
            {
                let _ = zstd_level;
                unreachable!();
            }
        } else {
            (raw, PayloadKind::Raw)
        };

        out.write_all(&payload)?;
        let payload_len: u64 = payload.len() as u64;

        entries.push(Entry {
            path: logical,
            payload_offset,
            payload_len,
            raw_len,
            raw_hash,
            payload_kind: kind,
        });
    }

    let index_offset = out.stream_position()?;
    let mut index_hasher = Hasher::new();

    let mut index_buf: Vec<u8> = Vec::new();
    index_buf.extend_from_slice(&MAGIC);
    index_buf.extend_from_slice(&(entries.len() as u32).to_le_bytes());

    for e in &entries {
        let p = e.path.as_bytes();
        if p.len() > u16::MAX as usize {
            return Err(PakError::Invalid(format!("path too long: {}", e.path)));
        }
        index_buf.extend_from_slice(&(p.len() as u16).to_le_bytes());
        index_buf.extend_from_slice(p);
        index_buf.extend_from_slice(&e.payload_offset.to_le_bytes());
        index_buf.extend_from_slice(&e.payload_len.to_le_bytes());
        index_buf.extend_from_slice(&e.raw_len.to_le_bytes());
        index_buf.push(e.payload_kind as u8);
        index_buf.extend_from_slice(&e.raw_hash);
    }

    index_hasher.update(&index_buf);
    let index_hash_full: [u8; 32] = index_hasher.finalize().into();
    let index_hash_u32 = u32::from_le_bytes([
        index_hash_full[0],
        index_hash_full[1],
        index_hash_full[2],
        index_hash_full[3],
    ]);

    out.write_all(&index_buf)?;
    let index_len = out.stream_position()? - index_offset;

    out.write_all(&FOOTER_MAGIC)?;
    write_u64(&mut out, index_offset)?;
    write_u64(&mut out, index_len)?;
    write_u32(&mut out, index_hash_u32)?;
    write_u32(&mut out, 0)?;

    out.flush()?;
    Ok(())
}
