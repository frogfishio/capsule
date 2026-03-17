// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use capsule_lib::ascii_header::HeaderField;
use capsule_lib::{Encoding, Version};

#[derive(Debug, Deserialize)]
pub struct CapsuleSpec {
    /// Capsule version as 4 hex digits (e.g. "0001").
    #[serde(default = "default_version")]
    pub version: String,

    /// Encoding: "A", "B", or "C".
    #[serde(default = "default_encoding")]
    pub encoding: String,

    /// Payload file path.
    pub payload: PathBuf,

    /// Output capsule path.
    pub out: PathBuf,

    /// Header bytes file (required for encoding B/C).
    pub header_file: Option<PathBuf>,

    /// Header fields (encoding A only). Keys are case-sensitive.
    #[serde(default)]
    pub header: BTreeMap<String, String>,
}

fn default_version() -> String {
    "0001".to_string()
}

fn default_encoding() -> String {
    "A".to_string()
}

pub fn load_spec(path: &Path) -> Result<CapsuleSpec> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read spec {}", path.display()))?;
    let mut spec: CapsuleSpec = toml::from_str(&text)
        .with_context(|| format!("parse TOML spec {}", path.display()))?;

    // Resolve paths relative to the spec file location.
    let base_dir = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    if spec.payload.is_relative() {
        spec.payload = base_dir.join(&spec.payload);
    }

    if spec.out.is_relative() {
        spec.out = base_dir.join(&spec.out);
    }

    if let Some(h) = &spec.header_file {
        if h.is_relative() {
            spec.header_file = Some(base_dir.join(h));
        }
    }

    Ok(spec)
}

pub fn parse_version_hex(s: &str) -> Result<Version> {
    if s.len() != 4 {
        anyhow::bail!("version must be 4 hex digits");
    }
    let v = u16::from_str_radix(s, 16).context("invalid version hex")?;
    if v == 0 {
        anyhow::bail!("version 0000 is reserved");
    }
    Ok(Version(v))
}

pub fn parse_encoding(s: &str) -> Result<Encoding> {
    match s.as_bytes() {
        b"A" => Ok(Encoding::Ascii),
        b"B" => Ok(Encoding::Base64),
        b"C" => Ok(Encoding::Cbor),
        _ => anyhow::bail!("encoding must be A, B, or C"),
    }
}

pub fn header_fields_from_map(map: &BTreeMap<String, String>) -> Vec<HeaderField> {
    // Deterministic: BTreeMap iterates keys in sorted order.
    map.iter()
        .map(|(k, v)| HeaderField {
            key: k.clone(),
            value: v.clone(),
        })
        .collect()
}
