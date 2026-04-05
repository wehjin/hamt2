use crate::db::component::key::KEY_MAX_EID;
use crate::db::Ein;
use crate::space::Space;
use crate::trie::mem::value::MemValue;
use crate::trie::SpaceTrie;
use crate::{QueryError, TransactError};

pub struct MaxEid {
    start: Ein,
    current: Ein,
}

impl MaxEid {
    pub fn new(eid: Ein) -> Self {
        Self {
            start: eid,
            current: eid,
        }
    }
    pub async fn read<T: Space>(trie: &SpaceTrie<T>) -> Result<Self, QueryError> {
        if let Some(MemValue::U32(value)) = trie.query_value(KEY_MAX_EID).await? {
            Ok(Self::new(Ein(value as i32)))
        } else {
            Ok(Self::new(Ein(0)))
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
    pub async fn write<T: Space>(self, trie: SpaceTrie<T>) -> Result<SpaceTrie<T>, TransactError> {
        let trie = if self.current > self.start {
            trie.insert(KEY_MAX_EID, MemValue::from(self.current.to_i32() as u32))
                .await?
        } else {
            trie
        };
        Ok(trie)
    }
}
