use crate::db::types::key::KEY_MAX_EID;
use sky_types::trie::*;
use sky_types::db::Ein;
use sky_types::db::{QueryError, TransactError};
use sky_types::storage::{BaseStore, TrieEdit, TrieLoad};

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
    pub async fn read<S: BaseStore>(trie: &TrieLoad<S>) -> Result<Self, QueryError> {
        if let Some(TrieValue::U32(value)) = trie.query(KEY_MAX_EID).await? {
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
    pub async fn write<S: BaseStore>(self, trie: &mut TrieEdit<S>) -> Result<(), TransactError> {
        if self.current > self.start {
            trie.insert(KEY_MAX_EID, TrieValue::from(self.current.to_i32() as u32))
                .await?;
        }
        Ok(())
    }
}
