pub fn universal(bytes: impl AsRef<[u8]>, level: u32) -> u32 {
    let bytes = bytes.as_ref();
    let level = level as u64;
    let mut a: u128 = 31415;
    const B: u128 = 27183;
    let mut hash: u128 = 0;
    for i in 0..bytes.len() {
        hash = a.wrapping_mul(hash).wrapping_mul(level as u128) + bytes[i] as u128;
        a = a.wrapping_mul(B);
    }
    hash as u32
}
