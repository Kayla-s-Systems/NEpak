#![forbid(unsafe_code)]

/// NEPAK v1 header magic.
pub const MAGIC: [u8; 8] = *b"NEPAK\x01\x00\x00";

/// NEPAK v1 footer magic.
pub const FOOTER_MAGIC: [u8; 8] = *b"NEPAKEND";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum PayloadKind {
    Raw = 0,
    Zstd = 1,
}

#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub path: String,
    pub payload_offset: u64,
    pub payload_len: u64,
    pub raw_len: u64,
    pub raw_hash: [u8; 32],
    pub payload_kind: PayloadKind,
}

/// Public view of a pak entry (for GUI tooling, inspectors, etc.).
#[derive(Debug, Clone)]
pub struct EntryInfo {
    pub path: String,
    pub payload_offset: u64,
    pub payload_len: u64,
    pub raw_len: u64,
    /// "raw" or "zstd"
    pub payload_kind: &'static str,
    /// Blake3 hash (hex) of the raw, uncompressed bytes.
    pub raw_hash_hex: String,
}