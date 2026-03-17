pub fn compute_crc32_iso_hdlc(body: &[u8]) -> u32 {
    crc32fast::hash(body)
}
