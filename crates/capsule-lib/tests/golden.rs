use std::fs;
use std::path::PathBuf;

use capsule_lib::ascii_header::HeaderField;
use capsule_lib::{Capsule, Encoding, ParseOptions, Prelude};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn parse_verified(bytes: &[u8]) -> capsule_lib::CapsuleResult<capsule_lib::CapsuleDecoded> {
    Capsule::parse_with_options(
        bytes,
        ParseOptions {
            verify_crc: true,
            validate_encoding: true,
        },
    )
}

fn assert_canonical_roundtrip(bytes: &[u8]) {
    let decoded = parse_verified(bytes).unwrap();

    // Reserialize from the encoded blocks exactly as stored.
    let capsule = capsule_lib::Capsule {
        prelude: Prelude {
            version: decoded.prelude.version,
            encoding: decoded.prelude.encoding,
            header_len: 0,
            body_crc: 0,
        },
        header_encoded: decoded.header_encoded,
        payload_encoded: decoded.payload_encoded,
    };

    let reserialized = capsule.to_bytes().unwrap();
    assert_eq!(bytes, reserialized);
}

#[test]
fn golden_ascii_fixture_parses_and_roundtrips() {
    let bytes = fs::read(fixture_path("ascii.capsule")).unwrap();
    let decoded = parse_verified(&bytes).unwrap();

    assert_eq!(decoded.prelude.version.0, 0x0001);
    assert_eq!(decoded.prelude.encoding, Encoding::Ascii);
    assert_eq!(decoded.payload_decoded, b"hello\n");

    assert_eq!(
        decoded.header_fields,
        Some(vec![
            HeaderField {
                key: "dialect".to_string(),
                value: "example/1".to_string(),
            },
            HeaderField {
                key: "id".to_string(),
                value: "123".to_string(),
            },
        ])
    );

    assert_canonical_roundtrip(&bytes);
}

#[test]
fn golden_base64_fixture_parses_and_roundtrips() {
    let bytes = fs::read(fixture_path("base64.capsule")).unwrap();
    let decoded = parse_verified(&bytes).unwrap();

    assert_eq!(decoded.prelude.encoding, Encoding::Base64);
    assert_eq!(decoded.header_decoded, b"hdr");
    assert_eq!(decoded.payload_decoded, vec![0x7F, b'E', b'L', b'F', 0x00, 0x01, 0x02, 0xFF]);

    assert_canonical_roundtrip(&bytes);
}

#[test]
fn golden_cbor_fixture_parses_and_roundtrips() {
    let bytes = fs::read(fixture_path("cbor.capsule")).unwrap();
    let decoded = parse_verified(&bytes).unwrap();

    assert_eq!(decoded.prelude.encoding, Encoding::Cbor);
    assert!(decoded.header_decoded.is_empty());
    assert_eq!(decoded.payload_decoded, vec![0x01, 0x61, 0x61]);

    assert_canonical_roundtrip(&bytes);
}

#[test]
fn crc_mismatch_is_rejected_when_verifying() {
    let mut bytes = fs::read(fixture_path("ascii.capsule")).unwrap();
    *bytes.last_mut().unwrap() ^= 0xFF;

    let err = Capsule::parse_with_options(
        &bytes,
        ParseOptions {
            verify_crc: true,
            validate_encoding: true,
        },
    )
    .unwrap_err();

    assert!(matches!(err, capsule_lib::CapsuleError::CrcMismatch { .. }));
}
