// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::error::{CapsuleError, CapsuleResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderField {
    pub key: String,
    pub value: String,
}

fn is_allowed_key_byte(b: u8) -> bool {
    matches!(b,
        b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'_'
            | b'-'
            | b'.'
    )
}

pub fn parse_ascii_header_kv(header_bytes: &[u8]) -> CapsuleResult<Vec<HeaderField>> {
    for (i, &b) in header_bytes.iter().enumerate() {
        if b > 0x7F {
            return Err(CapsuleError::NonAsciiByte { which: "header", offset: i });
        }
    }

    let mut fields: Vec<HeaderField> = Vec::new();
    let mut seen_keys = std::collections::BTreeSet::<String>::new();

    for (line_index, line) in header_bytes.split(|&b| b == b'\n').enumerate() {
        if line.is_empty() {
            continue;
        }

        let Some(eq_pos) = line.iter().position(|&b| b == b'=') else {
            return Err(CapsuleError::InvalidAsciiHeader(format!(
                "line {line_index} is non-empty but contains no '='"
            )));
        };

        if eq_pos == 0 {
            return Err(CapsuleError::InvalidAsciiHeader(format!(
                "line {line_index} has empty key"
            )));
        }

        let key_bytes = &line[..eq_pos];
        let value_bytes = &line[eq_pos + 1..];

        if key_bytes.iter().any(|&b| !is_allowed_key_byte(b)) {
            return Err(CapsuleError::InvalidAsciiHeader(format!(
                "line {line_index} key contains invalid characters"
            )));
        }

        let key = String::from_utf8(key_bytes.to_vec()).map_err(|e| {
            CapsuleError::InvalidAsciiHeader(format!("line {line_index} key is not valid UTF-8: {e}"))
        })?;
        let value = String::from_utf8(value_bytes.to_vec()).map_err(|e| {
            CapsuleError::InvalidAsciiHeader(format!("line {line_index} value is not valid UTF-8: {e}"))
        })?;

        if !seen_keys.insert(key.clone()) {
            return Err(CapsuleError::InvalidAsciiHeader(format!(
                "duplicate key '{key}'"
            )));
        }

        fields.push(HeaderField { key, value });
    }

    Ok(fields)
}

pub fn encode_ascii_header_kv(fields: &[HeaderField]) -> CapsuleResult<Vec<u8>> {
    let mut out = Vec::new();
    let mut seen_keys = std::collections::BTreeSet::<&str>::new();

    for field in fields {
        if field.key.is_empty() {
            return Err(CapsuleError::InvalidAsciiHeader("empty key".to_string()));
        }

        if !seen_keys.insert(field.key.as_str()) {
            return Err(CapsuleError::InvalidAsciiHeader(format!(
                "duplicate key '{}'",
                field.key
            )));
        }

        if field.key.as_bytes().iter().any(|&b| !is_allowed_key_byte(b)) {
            return Err(CapsuleError::InvalidAsciiHeader(format!(
                "invalid key '{}'",
                field.key
            )));
        }

        if field.key.as_bytes().iter().any(|&b| b > 0x7F) || field.value.as_bytes().iter().any(|&b| b > 0x7F) {
            return Err(CapsuleError::InvalidAsciiHeader(
                "non-ASCII key/value".to_string(),
            ));
        }

        out.extend_from_slice(field.key.as_bytes());
        out.push(b'=');
        out.extend_from_slice(field.value.as_bytes());
        out.push(b'\n');
    }

    Ok(out)
}
