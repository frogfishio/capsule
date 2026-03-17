use thiserror::Error;

pub type CapsuleResult<T> = Result<T, CapsuleError>;

#[derive(Debug, Error)]
pub enum CapsuleError {
    #[error("file too short: expected at least 24 bytes, got {0}")]
    FileTooShort(usize),

    #[error("invalid magic")]
    InvalidMagic,

    #[error("invalid version field")]
    InvalidVersionField,

    #[error("reserved version 0000 is not allowed")]
    ReservedVersion,

    #[error("invalid encoding field")]
    InvalidEncodingField,

    #[error("invalid header length field")]
    InvalidHeaderLengthField,

    #[error("invalid body CRC field")]
    InvalidBodyCrcField,

    #[error("declared header length {declared} exceeds available bytes {available}")]
    HeaderLengthExceedsAvailable { declared: usize, available: usize },

    #[error("CRC mismatch: declared {declared:08X} computed {computed:08X}")]
    CrcMismatch { declared: u32, computed: u32 },

    #[error("non-ASCII byte found in {which} at offset {offset}")]
    NonAsciiByte { which: &'static str, offset: usize },

    #[error("invalid base64 in {which}: {message}")]
    InvalidBase64 { which: &'static str, message: String },

    #[error("invalid CBOR in {which}: {message}")]
    InvalidCbor { which: &'static str, message: String },

    #[error("invalid ASCII header: {0}")]
    InvalidAsciiHeader(String),
}
