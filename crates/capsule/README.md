<!-- SPDX-FileCopyrightText: 2026 Alexander R. Croft -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# capsule-cli

Command-line tool for packing, inspecting, verifying, and unpacking Capsule container files.

- Container format spec: see the repository’s SPEC.md
- Library crate: `capsule-lib`

## Install

From crates.io:

- `cargo install capsule-cli`

From source:

- `cargo install --path crates/capsule`

## Usage

- `capsule --help`

Global flags:

- `--version`: prints the tool version
- `--license`: prints copyright/license text

Common commands:

- `capsule pack --out out.capsule --payload ./payload.bin --encoding A --header dialect=example/1 --header id=123`
- `capsule info ./out.capsule`
- `capsule verify ./out.capsule`
- `capsule unpack ./out.capsule --out-payload payload.bin`

## Notes

- Encoding `A` treats the payload as an opaque byte blob and supports ASCII header fields in `key=value` form.
- Encodings `B` and `C` accept raw header bytes via `--header-file`.
