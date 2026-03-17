// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod ascii_header;
pub mod capsule;
pub mod crc;
pub mod encoding;
pub mod error;

pub use crate::capsule::{Capsule, CapsuleDecoded, ParseOptions, Prelude, Version};
pub use crate::encoding::Encoding;
pub use crate::error::{CapsuleError, CapsuleResult};
