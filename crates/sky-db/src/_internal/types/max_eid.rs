use crate::_internal::KEY_MAX_EID;
use crate::Ein;
use crate::trie::*;
use crate::trie::SkyTrieMut;
use crate::trie::SkyTrie;
use crate::{QueryError, TransactError};

pub struct MaxEid {
    start: Ein,
    current: Ein,
}

impl MaxEid {
    fn new(eid: Ein) -> Self {
        Self {
            start: eid,
            current: eid,
        }
    }
    pub async fn read(trie: &SkyTrie) -> Result<Self, QueryError> {
        if let Some(TrieValue::U32(value)) = trie.query(KEY_MAX_EID) {
            Ok(Self::new(Ein(value as i32)))
        } else {
            Ok(Self::new(Ein::DB_MAX))
        }
    }
    pub fn take(&mut self, count: usize) -> Vec<Ein> {
        let mut taken = Vec::new();
        for _ in 0..count {
            taken.push(self.current);
            self.current += 1;
        }
        taken
    }
    pub async fn write(self, trie: &mut SkyTrieMut) -> Result<(), TransactError> {
        if self.current > self.start {
            trie.insert(KEY_MAX_EID, TrieValue::from(self.current.to_i32() as u32))
                .await;
        }
        Ok(())
    }
}
