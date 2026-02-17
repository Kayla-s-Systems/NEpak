#![forbid(unsafe_code)]

use std::io::{Read, Write};

use crate::pak::error::PakResult;

pub fn write_u32(w: &mut dyn Write, v: u32) -> PakResult<()> {
    w.write_all(&v.to_le_bytes())?;
    Ok(())
}

pub fn write_u64(w: &mut dyn Write, v: u64) -> PakResult<()> {
    w.write_all(&v.to_le_bytes())?;
    Ok(())
}

pub fn read_exact<const N: usize>(r: &mut dyn Read) -> PakResult<[u8; N]> {
    let mut buf = [0u8; N];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

pub fn read_u8(r: &mut dyn Read) -> PakResult<u8> {
    Ok(read_exact::<1>(r)?[0])
}

pub fn read_u16(r: &mut dyn Read) -> PakResult<u16> {
    Ok(u16::from_le_bytes(read_exact::<2>(r)?))
}

pub fn read_u32(r: &mut dyn Read) -> PakResult<u32> {
    Ok(u32::from_le_bytes(read_exact::<4>(r)?))
}

pub fn read_u64(r: &mut dyn Read) -> PakResult<u64> {
    Ok(u64::from_le_bytes(read_exact::<8>(r)?))
}

pub fn hex32(v: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = [0u8; 64];
    for (i, b) in v.iter().copied().enumerate() {
        out[i * 2] = HEX[(b >> 4) as usize];
        out[i * 2 + 1] = HEX[(b & 0xF) as usize];
    }
    String::from_utf8_lossy(&out).into_owned()
}