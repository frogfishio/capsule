// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: GPL-3.0-or-later

pub fn compute_crc32_iso_hdlc(body: &[u8]) -> u32 {
    crc32fast::hash(body)
}
