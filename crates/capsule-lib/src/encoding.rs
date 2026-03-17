use crate::error::{CapsuleError, CapsuleResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Ascii,
    Base64,
    Cbor,
}

impl Encoding {
    pub fn from_byte(b: u8) -> CapsuleResult<Self> {
        match b {
            b'A' => Ok(Self::Ascii),
            b'B' => Ok(Self::Base64),
            b'C' => Ok(Self::Cbor),
            _ => Err(CapsuleError::InvalidEncodingField),
        }
    }

    pub fn to_byte(self) -> u8 {
        match self {
            Self::Ascii => b'A',
            Self::Base64 => b'B',
            Self::Cbor => b'C',
        }
    }
}
