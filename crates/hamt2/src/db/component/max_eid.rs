use crate::db::component::key::KEY_MAX_EID;
use crate::db::Ein;
use crate::trie::base_storage::BaseStorageReadWrite;
use crate::trie::mem::value::MemValue;
use crate::trie::{Trie, TrieQuery};
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
    pub async fn read<S: BaseStorageReadWrite>(trie: &Trie<S>) -> Result<Self, QueryError> {
        if let Some(MemValue::U32(value)) = trie.query_value(KEY_MAX_EID).await? {
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
    pub async fn write<S: BaseStorageReadWrite>(self, trie: Trie<S>) -> Result<Trie<S>, TransactError> {
        let trie = if self.current > self.start {
            trie.insert(KEY_MAX_EID, MemValue::from(self.current.to_i32() as u32))
                .await?
        } else {
            trie
        };
        Ok(trie)
    }
}
