<!-- SPDX-FileCopyrightText: 2026 Alexander R. Croft -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# TODO

- [x] Build a Rust library (capsule-lib) that can read/write Capsule files per SPEC.md:
	parse/emit the 24-byte prelude; handle header length exactly in encoded bytes;
	compute/verify Body CRC; support encodings A (ASCII), B (Base64), C (CBOR).
	Also: when Encoding = A, parse and emit the ASCII header `key=value\n` block into a
	portable structure (and provide the encoded header bytes). The library MUST treat
	the payload as an opaque binary blob (it may be an ELF executable, etc.) and simply
	return it to callers after container decoding.

- [x] Build a CLI binary (capsule) to pack/unpack/inspect capsules.
	Default workflow should not require a spec file (e.g., `capsule pack ...`,
	`capsule verify`, `capsule info`).
	The tool MUST treat the payload as an opaque binary blob and MUST NOT impose any
	internal structure beyond Capsule framing and container decoding.

- [x] Define optional `capsule.toml` as a packaging specification for complex cases
	(e.g., header field population rules, input mapping).
	The CLI should still work without `capsule.toml`; the TOML is for when the
	packaging rules need to be repeatable and declarative.