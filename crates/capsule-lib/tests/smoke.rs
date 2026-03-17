use capsule_lib::ascii_header::HeaderField;
use capsule_lib::{Capsule, Encoding, Version};

#[test]
fn roundtrip_ascii_header_and_payload() {
    let capsule = Capsule::from_decoded(
        Version(0x0001),
        Encoding::Ascii,
        Some(&[
            HeaderField { key: "dialect".to_string(), value: "example/1".to_string() },
            HeaderField { key: "id".to_string(), value: "123".to_string() },
        ]),
        &[],
        b"hello\n",
    )
    .unwrap();

    let bytes = capsule.to_bytes().unwrap();
    let decoded = Capsule::parse(&bytes).unwrap();

    assert_eq!(decoded.prelude.version.0, 0x0001);
    assert_eq!(decoded.prelude.encoding, Encoding::Ascii);
    assert_eq!(decoded.payload_decoded, b"hello\n");

    let fields = decoded.header_fields.unwrap();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].key, "dialect");
    assert_eq!(fields[0].value, "example/1");
}

#[test]
fn roundtrip_base64_payload_binary_blob() {
    let payload = [0x7F, b'E', b'L', b'F', 0x00, 0x01, 0x02, 0xFF];

    let capsule = Capsule::from_decoded(
        Version(0x0001),
        Encoding::Base64,
        None,
        b"hdr",
        &payload,
    )
    .unwrap();

    let bytes = capsule.to_bytes().unwrap();
    let decoded = Capsule::parse(&bytes).unwrap();

    assert_eq!(decoded.prelude.encoding, Encoding::Base64);
    assert_eq!(decoded.header_decoded, b"hdr");
    assert_eq!(decoded.payload_decoded, payload);
}

#[test]
fn accepts_cbor_sequence_payload() {
    // CBOR sequence: unsigned(1) followed by text("a")
    let cbor_sequence = [0x01, 0x61, 0x61];

    let capsule = Capsule::from_decoded(Version(0x0001), Encoding::Cbor, None, &[], &cbor_sequence).unwrap();
    let bytes = capsule.to_bytes().unwrap();

    let decoded = Capsule::parse_with_options(
        &bytes,
        capsule_lib::ParseOptions {
            verify_crc: true,
            validate_encoding: true,
        },
    )
    .unwrap();

    assert_eq!(decoded.prelude.encoding, Encoding::Cbor);
    assert_eq!(decoded.payload_decoded, cbor_sequence);
}
