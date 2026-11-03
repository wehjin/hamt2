pub const KEY_EAVT: i32 = 1;
pub const KEY_AEVT: i32 = 2;
pub const KEY_VAL_TABLE: i32 = 4;

/// The mask for 31-bit, non-negative entity/value ids. `i32::MAX` keeps every
/// hash-derived id a valid, non-negative trie key.
pub const ID_MASK: u32 = i32::MAX as u32;
