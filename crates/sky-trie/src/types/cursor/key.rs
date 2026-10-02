#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TrieKey(i32);

impl TrieKey {
    pub const MASK: u32 = 0x3FFF_FFFF;
}

impl From<i32> for TrieKey {
    fn from(value: i32) -> Self {
        debug_assert!(value >= 0, "Attempted to create key with high bit");
        Self(value)
    }
}

impl Into<i32> for TrieKey {
    fn into(self) -> i32 {
        self.0
    }
}

pub fn key(value: impl Into<TrieKey>) -> TrieKey {
    value.into()
}
