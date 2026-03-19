use crate::hamt::space;
use crate::hamt::trie::mem::value::MemValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrieValue {
    Mem(MemValue),
    Space(space::ValueAddr),
}

impl TrieValue {
    pub fn u32(v: u32) -> Self {
        assert_eq!(
            0,
            v & 0x80000000,
            "TrieValue u32 must not have the high bit set"
        );
        TrieValue::Mem(MemValue::U32(v))
    }
}
