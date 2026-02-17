#![forbid(unsafe_code)]

use blake3::Hasher;
use std::cmp::Ordering;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use crate::pak::error::{PakError, PakResult};
use crate::pak::format::{Entry, PayloadKind, FOOTER_MAGIC, MAGIC};
use crate::pak::io::{read_exact, read_u16, read_u32, read_u64, read_u8};

pub(crate) fn read_footer(file: &mut File) -> PakResult<(u64, u64, u32)> {
    let size = file.metadata()?.len();
    let footer_len = 8 + 8 + 8 + 4 + 4;
    if size < footer_len {
        return Err(PakError::Invalid("file too small".into()));
    }
    file.seek(SeekFrom::End(-(footer_len as i64)))?;

    let magic = read_exact::<8>(file)?;
    if magic != FOOTER_MAGIC {
        return Err(PakError::Invalid("bad footer magic".into()));
    }

    let index_offset = read_u64(file)?;
    let index_len = read_u64(file)?;
    let index_hash = read_u32(file)?;
    let _reserved = read_u32(file)?;

    Ok((index_offset, index_len, index_hash))
}

pub(crate) fn read_index(file: &mut File) -> PakResult<Vec<Entry>> {
    let (index_offset, index_len, index_hash_u32) = read_footer(file)?;
    let file_len = file.metadata()?.len();

    if index_offset + index_len > file_len {
        return Err(PakError::Invalid("index outside file".into()));
    }

    file.seek(SeekFrom::Start(index_offset))?;
    let mut index_buf = vec![0u8; index_len as usize];
    file.read_exact(&mut index_buf)?;

    let mut hasher = Hasher::new();
    hasher.update(&index_buf);
    let full: [u8; 32] = hasher.finalize().into();
    let got_u32 = u32::from_le_bytes([full[0], full[1], full[2], full[3]]);
    if got_u32 != index_hash_u32 {
        return Err(PakError::Invalid("index hash mismatch".into()));
    }

    let mut cur = std::io::Cursor::new(index_buf);

    let magic = read_exact::<8>(&mut cur)?;
    if magic != MAGIC {
        return Err(PakError::Invalid("bad index magic".into()));
    }

    let count = read_u32(&mut cur)? as usize;
    let mut out: Vec<Entry> = Vec::with_capacity(count);

    for _ in 0..count {
        let path_len = read_u16(&mut cur)? as usize;
        let mut path_bytes = vec![0u8; path_len];
        cur.read_exact(&mut path_bytes)?;
        let path = String::from_utf8(path_bytes)
            .map_err(|_| PakError::Invalid("path is not utf8".into()))?;

        let payload_offset = read_u64(&mut cur)?;
        let payload_len = read_u64(&mut cur)?;
        let raw_len = read_u64(&mut cur)?;
        let kind = match read_u8(&mut cur)? {
            0 => PayloadKind::Raw,
            1 => PayloadKind::Zstd,
            other => return Err(PakError::Invalid(format!("unknown payload kind {other}"))),
        };
        let raw_hash = read_exact::<32>(&mut cur)?;

        out.push(Entry {
            path,
            payload_offset,
            payload_len,
            raw_len,
            raw_hash,
            payload_kind: kind,
        });
    }

    for w in out.windows(2) {
        if w[0].path.as_bytes().cmp(w[1].path.as_bytes()) == Ordering::Greater {
            return Err(PakError::Invalid("index is not sorted".into()));
        }
    }

    Ok(out)
}