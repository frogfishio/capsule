<!-- SPDX-FileCopyrightText: 2026 Alexander R. Croft -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Capsule

Capsule is a tiny container format (see SPEC.md) plus a Rust implementation:

- `capsule-lib`: parse/write Capsule files
- `capsule-cli`: CLI for packing, inspecting, and verifying Capsule files

## Build

- `cargo build`
- `cargo test`

## Library

The Rust library is `capsule-lib`.

- Library README: [crates/capsule-lib/README.md](crates/capsule-lib/README.md)
- API entrypoint: [crates/capsule-lib/src/lib.rs](crates/capsule-lib/src/lib.rs)

Quick integration (path dependency):

```toml
[dependencies]
capsule-lib = { path = "path/to/capsule/crates/capsule-lib" }
```

## CLI

Global flags:

- `--version`: prints the tool version as `$(cat VERSION)+build.$(cat BUILD)`
- `--license`: prints copyright/license text

Pack an ASCII-encoded capsule:

- `cargo run -p capsule-cli -- pack --out out.capsule --payload ./payload.txt --encoding A --header dialect=example/1 --header id=123`

Pack a binary payload using Base64 encoding:

- `cargo run -p capsule-cli -- pack --out out.capsule --payload ./payload.bin --encoding B --header-file ./header.bin`

Pack using a `capsule.toml` spec (all fields are deterministic; CLI flags override spec values):

- `cargo run -p capsule-cli -- pack --spec ./capsule.toml`

Example `capsule.toml` (ASCII encoding):

```toml
encoding = "A"
payload = "./payload.bin"
out = "./out.capsule"

[header]
dialect = "example/1"
id = "123"
```

Example `capsule.toml` (Base64/CBOR encodings use opaque header bytes):

```toml
encoding = "B"
payload = "./payload.bin"
header_file = "./header.bin"
out = "./out.capsule"
```

Inspect a capsule (optionally verify CRC / encoding validity):

- `cargo run -p capsule-cli -- info ./out.capsule`
- `cargo run -p capsule-cli -- info --verify ./out.capsule`

Verify a capsule:

- `cargo run -p capsule-cli -- verify ./out.capsule`

Unpack (decode) a capsule to bytes (payload is always treated as an opaque blob):

- `cargo run -p capsule-cli -- unpack ./out.capsule --out-payload ./payload.out`
- `cargo run -p capsule-cli -- unpack ./out.capsule --out-payload ./payload.out --out-header ./header.out --verify`

Unpack raw encoded bytes exactly as stored (no decoding):

- `cargo run -p capsule-cli -- unpack ./out.capsule --out-payload ./payload.encoded --out-header ./header.encoded --raw`