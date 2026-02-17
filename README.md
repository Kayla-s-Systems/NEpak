# nepak — PakBuilder for NewEngine

CLI tool to build deterministic `.pak` containers (NEPAK v1).

## Install (workspace)

Add as a workspace member and build:

```bash
cargo build -p nepak --release
```

## Usage

### Interactive wizard (simple UI)

```bash
nepak ui
```

### Build

```bash
nepak build --input ./assets --output ./assets.pak --prefix assets --compress --zstd-level 6
```

* `--prefix` is an in-pak mount prefix (optional). Useful if your engine expects `assets/...` logical paths.
* `--exclude` is a repeatable simple substring filter on normalized paths.

### List

```bash
nepak list --pak ./assets.pak
nepak list --pak ./assets.pak --verbose
```

### Extract

```bash
nepak extract --pak ./assets.pak --output ./out
```

### Verify

```bash
nepak verify --pak ./assets.pak
```

## NEPAK v1 format (spec)

The file layout is designed to be simple and robust:

* Header: 8-byte magic `NEPAK\x01\x00\x00`
* Payload blobs: concatenated file payloads (raw or zstd)
* Index:
  * magic (same as header)
  * `u32 entry_count`
  * entries (sorted by path):
    * `u16 path_len` + `path bytes (utf-8, '/' separators)`
    * `u64 payload_offset`
    * `u64 payload_len`
    * `u64 raw_len`
    * `u8 payload_kind` (0=raw, 1=zstd)
    * `raw_hash[32]` (blake3 of uncompressed data)
* Footer:
  * 8-byte magic `NEPAKEND`
  * `u64 index_offset`
  * `u64 index_len`
  * `u32 index_hash` (blake3(index_bytes) truncated to u32)
  * `u32 reserved`

Determinism:

* paths are normalized to `/`
* entries are sorted lexicographically by path bytes

