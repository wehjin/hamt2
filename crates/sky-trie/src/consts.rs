/// The 30-bit mask for trie keys. The top two bits of a `KeyValue` key tag
/// Int/Subtrie/Bytes, leaving 30 bits for the key itself.
pub const TRIE_KEY_MASK: u32 = 0x3FFF_FFFF;
