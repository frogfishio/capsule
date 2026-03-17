use crate::ascii_header::{encode_ascii_header_kv, parse_ascii_header_kv, HeaderField};
use crate::crc::compute_crc32_iso_hdlc;
use crate::encoding::Encoding;
use crate::error::{CapsuleError, CapsuleResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(pub u16);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prelude {
    pub version: Version,
    pub encoding: Encoding,
    pub header_len: u16,
    pub body_crc: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capsule {
    pub prelude: Prelude,
    pub header_encoded: Vec<u8>,
    pub payload_encoded: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapsuleDecoded {
    pub prelude: Prelude,
    pub header_encoded: Vec<u8>,
    pub payload_encoded: Vec<u8>,
    pub header_decoded: Vec<u8>,
    pub payload_decoded: Vec<u8>,
    pub header_fields: Option<Vec<HeaderField>>,
}

#[derive(Debug, Clone, Copy)]
pub struct ParseOptions {
    pub verify_crc: bool,
    pub validate_encoding: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self { verify_crc: true, validate_encoding: true }
    }
}

fn parse_upper_hex_u16(bytes: &[u8]) -> Option<u16> {
    if bytes.len() != 4 { return None; }
    let mut v: u16 = 0;
    for &b in bytes {
        let d = match b {
            b'0'..=b'9' => b - b'0',
            b'A'..=b'F' => b - b'A' + 10,
            _ => return None,
        };
        v = (v << 4) | d as u16;
    }
    Some(v)
}

fn parse_upper_hex_u32(bytes: &[u8]) -> Option<u32> {
    if bytes.len() != 8 { return None; }
    let mut v: u32 = 0;
    for &b in bytes {
        let d = match b {
            b'0'..=b'9' => b - b'0',
            b'A'..=b'F' => b - b'A' + 10,
            _ => return None,
        };
        v = (v << 4) | d as u32;
    }
    Some(v)
}

fn ensure_ascii(which: &'static str, bytes: &[u8]) -> CapsuleResult<()> {
    for (i, &b) in bytes.iter().enumerate() {
        if b > 0x7F {
            return Err(CapsuleError::NonAsciiByte { which, offset: i });
        }
    }
    Ok(())
}

fn decode_base64(which: &'static str, bytes: &[u8]) -> CapsuleResult<Vec<u8>> {
    // data-encoding rejects non-alphabet chars including whitespace and enforces RFC4648 padding.
    data_encoding::BASE64
        .decode(bytes)
        .map_err(|e| CapsuleError::InvalidBase64 { which, message: e.to_string() })
}

fn validate_cbor(which: &'static str, bytes: &[u8]) -> CapsuleResult<()> {
    use serde::de::IgnoredAny;

    if bytes.is_empty() {
        return Ok(());
    }

    // Accept a CBOR sequence (0..N concatenated data items). This performs
    // well-formedness validation only and does not impose any schema.
    let mut it = serde_cbor::Deserializer::from_slice(bytes).into_iter::<IgnoredAny>();
    while let Some(item) = it.next() {
        item.map_err(|e| CapsuleError::InvalidCbor {
            which,
            message: e.to_string(),
        })?;
    }

    if it.byte_offset() != bytes.len() {
        return Err(CapsuleError::InvalidCbor {
            which,
            message: "trailing bytes after CBOR sequence".to_string(),
        });
    }

    Ok(())
}

impl Capsule {
    pub fn parse(bytes: &[u8]) -> CapsuleResult<CapsuleDecoded> {
        Self::parse_with_options(bytes, ParseOptions::default())
    }

    pub fn parse_with_options(bytes: &[u8], options: ParseOptions) -> CapsuleResult<CapsuleDecoded> {
        if bytes.len() < 24 {
            return Err(CapsuleError::FileTooShort(bytes.len()));
        }

        if &bytes[..7] != b"CAPSULE" {
            return Err(CapsuleError::InvalidMagic);
        }

        let version_u16 = parse_upper_hex_u16(&bytes[7..11]).ok_or(CapsuleError::InvalidVersionField)?;
        if version_u16 == 0 {
            return Err(CapsuleError::ReservedVersion);
        }

        let encoding = Encoding::from_byte(bytes[11])?;
        let header_len = parse_upper_hex_u16(&bytes[12..16]).ok_or(CapsuleError::InvalidHeaderLengthField)?;
        let body_crc = parse_upper_hex_u32(&bytes[16..24]).ok_or(CapsuleError::InvalidBodyCrcField)?;

        let header_len_usize = header_len as usize;
        let available = bytes.len() - 24;
        if header_len_usize > available {
            return Err(CapsuleError::HeaderLengthExceedsAvailable { declared: header_len_usize, available });
        }

        let header_start = 24;
        let header_end = 24 + header_len_usize;
        let header_encoded = bytes[header_start..header_end].to_vec();
        let payload_encoded = bytes[header_end..].to_vec();

        if options.verify_crc {
            let computed = compute_crc32_iso_hdlc(&bytes[24..]);
            if computed != body_crc {
                return Err(CapsuleError::CrcMismatch { declared: body_crc, computed });
            }
        }

        let (header_decoded, payload_decoded, header_fields) = match encoding {
            Encoding::Ascii => {
                if options.validate_encoding {
                    ensure_ascii("header", &header_encoded)?;
                    ensure_ascii("payload", &payload_encoded)?;
                }
                let fields = parse_ascii_header_kv(&header_encoded)?;
                (header_encoded.clone(), payload_encoded.clone(), Some(fields))
            }
            Encoding::Base64 => {
                let header = decode_base64("header", &header_encoded)?;
                let payload = decode_base64("payload", &payload_encoded)?;
                (header, payload, None)
            }
            Encoding::Cbor => {
                if options.validate_encoding {
                    validate_cbor("header", &header_encoded)?;
                    validate_cbor("payload", &payload_encoded)?;
                }
                (header_encoded.clone(), payload_encoded.clone(), None)
            }
        };

        Ok(CapsuleDecoded {
            prelude: Prelude {
                version: Version(version_u16),
                encoding,
                header_len,
                body_crc,
            },
            header_encoded,
            payload_encoded,
            header_decoded,
            payload_decoded,
            header_fields,
        })
    }

    pub fn to_bytes(&self) -> CapsuleResult<Vec<u8>> {
        let header_len = self.header_encoded.len();
        if header_len > u16::MAX as usize {
            return Err(CapsuleError::InvalidHeaderLengthField);
        }

        let mut out = Vec::with_capacity(24 + header_len + self.payload_encoded.len());
        out.extend_from_slice(b"CAPSULE");

        let version = self.prelude.version.0;
        if version == 0 {
            return Err(CapsuleError::ReservedVersion);
        }

        out.extend_from_slice(format!("{:04X}", version).as_bytes());
        out.push(self.prelude.encoding.to_byte());
        out.extend_from_slice(format!("{:04X}", header_len as u16).as_bytes());

        let mut body = Vec::with_capacity(header_len + self.payload_encoded.len());
        body.extend_from_slice(&self.header_encoded);
        body.extend_from_slice(&self.payload_encoded);

        let crc = compute_crc32_iso_hdlc(&body);
        out.extend_from_slice(format!("{:08X}", crc).as_bytes());
        out.extend_from_slice(&body);

        Ok(out)
    }

    pub fn from_decoded(
        version: Version,
        encoding: Encoding,
        header_fields: Option<&[HeaderField]>,
        header_decoded: &[u8],
        payload_decoded: &[u8],
    ) -> CapsuleResult<Self> {
        let header_encoded = match encoding {
            Encoding::Ascii => {
                let fields = header_fields.ok_or_else(|| {
                    CapsuleError::InvalidAsciiHeader("missing header fields for ASCII encoding".to_string())
                })?;
                encode_ascii_header_kv(fields)?
            }
            Encoding::Base64 => data_encoding::BASE64.encode(header_decoded).into_bytes(),
            Encoding::Cbor => {
                validate_cbor("header", header_decoded)?;
                header_decoded.to_vec()
            }
        };

        let payload_encoded = match encoding {
            Encoding::Ascii => {
                ensure_ascii("payload", payload_decoded)?;
                payload_decoded.to_vec()
            }
            Encoding::Base64 => data_encoding::BASE64.encode(payload_decoded).into_bytes(),
            Encoding::Cbor => {
                validate_cbor("payload", payload_decoded)?;
                payload_decoded.to_vec()
            }
        };

        Ok(Self {
            prelude: Prelude {
                version,
                encoding,
                header_len: header_encoded.len() as u16,
                body_crc: 0,
            },
            header_encoded,
            payload_encoded,
        })
    }
}
