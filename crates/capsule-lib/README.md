<!-- SPDX-FileCopyrightText: 2026 Alexander R. Croft -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# capsule-lib

Rust library for reading and writing Capsule container files (see the repository’s SPEC.md).

Capsule is a framing/container format:

- It provides a fixed-width ASCII prelude + header block + payload block.
- The payload is always treated as an opaque byte blob at the Capsule layer.
- Encoding selection (A/B/C) defines how header/payload bytes are represented in the container.

This library is intentionally strict/deterministic:

- Header length is measured in *encoded bytes as stored*.
- CRC-32 verification is supported (enabled by default when parsing).
- ASCII header `key=value\n` parsing is provided when Encoding = A.

## Add As A Dependency

If you vendor this repo into a larger workspace:

```toml
[dependencies]
capsule-lib = { path = "path/to/capsule/crates/capsule-lib" }
```

If you depend on a git checkout (fill in the URL + revision you want):

```toml
[dependencies]
capsule-lib = { git = "<REPO_URL>", rev = "<GIT_SHA>", package = "capsule-lib" }
```

## Core Types

- `Capsule::parse(...)` / `Capsule::parse_with_options(...)` parse a full Capsule file from bytes.
- `Capsule::to_bytes()` serializes a `Capsule` back to bytes (recomputes CRC).
- `Capsule::from_decoded(...)` builds a Capsule from decoded header/payload inputs.
- `CapsuleDecoded` is the parse result (prelude + encoded + decoded bytes).
- `ParseOptions` controls CRC verification and encoding validation.

## Parsing Example

```rust
use std::fs;

use capsule_lib::{Capsule, ParseOptions, Encoding};

fn main() -> capsule_lib::CapsuleResult<()> {
    let bytes = fs::read("input.capsule").unwrap();

    // Strict mode (recommended for verification tooling).
    let decoded = Capsule::parse_with_options(
        &bytes,
        ParseOptions { verify_crc: true, validate_encoding: true },
    )?;

    match decoded.prelude.encoding {
        Encoding::Ascii => {
            // For Encoding=A: header_fields is populated and payload_decoded is ASCII bytes.
            if let Some(fields) = decoded.header_fields {
                for f in fields {
                    println!("{}={}", f.key, f.value);
                }
            }
            // Payload is still opaque at the Capsule layer.
            let payload: &[u8] = &decoded.payload_decoded;
            println!("payload len: {}", payload.len());
        }
        Encoding::Base64 => {
            // For Encoding=B: payload_decoded is Base64-decoded bytes.
            let payload: &[u8] = &decoded.payload_decoded;
            println!("decoded payload len: {}", payload.len());
        }
        Encoding::Cbor => {
            // For Encoding=C: this library does not interpret CBOR. When validation is enabled,
            // it validates well-formedness only (accepts a CBOR sequence).
            let payload: &[u8] = &decoded.payload_decoded;
            println!("cbor payload bytes: {}", payload.len());
        }
    }

    Ok(())
}
```

## Writing Example (Encoding A)

```rust
use capsule_lib::ascii_header::HeaderField;
use capsule_lib::{Capsule, Encoding, Version};

fn main() -> capsule_lib::CapsuleResult<()> {
    let header_fields = vec![
        HeaderField { key: "dialect".to_string(), value: "example/1".to_string() },
        HeaderField { key: "id".to_string(), value: "123".to_string() },
    ];

    let payload = b"hello\n";

    let capsule = Capsule::from_decoded(
        Version(0x0001),
        Encoding::Ascii,
        Some(&header_fields),
        &[],
        payload,
    )?;

    let bytes = capsule.to_bytes()?;
    std::fs::write("out.capsule", bytes).unwrap();

    Ok(())
}
```

## Validation Guidance

- `verify_crc=true` is recommended for `capsule verify`-style behavior.
- `validate_encoding=true` performs lexical validation only:
  - A: rejects non-ASCII bytes in header/payload
  - B: strict RFC 4648 Base64 decode
  - C: validates CBOR well-formedness and accepts a CBOR sequence (0..N items)

## Notes

- The Capsule container format version emitted by tooling is currently `0001`.
- `capsule-lib` treats payload bytes as opaque; higher-level parsers can interpret payload content as needed.
