use crate::db::types::key::KEY_MAX_EID;
use crate::trie::prelude::*;
use sky_types::db::{QueryError, TransactError};
use sky_types::db::Ein;
use sky_types::storage::ReadWriteStorage;

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
    pub async fn read<S: ReadWriteStorage + TrieBaseRead<Config = HandleTrieConfig>>(trie: &Trie<S>) -> Result<Self, QueryError> {
        if let Some(TrieValue::U32(value)) = trie.query_value(KEY_MAX_EID).await? {
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
    pub async fn write<S: ReadWriteStorage + TrieBaseRead<Config = HandleTrieConfig>>(
        self,
        trie: Trie<S>,
    ) -> Result<Trie<S>, TransactError> {
        let trie = if self.current > self.start {
            trie.insert(KEY_MAX_EID, TrieValue::from(self.current.to_i32() as u32))
                .await?
        } else {
            trie
        };
        Ok(trie)
    }
}
