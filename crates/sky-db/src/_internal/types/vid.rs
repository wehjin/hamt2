use crate::trie::TrieKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vid(i32);

impl Vid {
    pub fn from_id(i: i32) -> Self {
        Self(i)
    }
    pub fn to_id(&self) -> i32 {
        self.0
    }

    pub fn for_search(value: u32) -> Self {
        Self((value & TrieKey::MASK) as i32)
    }

    pub fn next_search(self) -> Self {
        let next = self.0 as u32 + 1;
        let rolled_next = if next & !TrieKey::MASK == 0 { next } else { 0 };
        Self(rolled_next as i32)
    }
}
