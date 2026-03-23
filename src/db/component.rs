use crate::db::Ent;
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::space::SpaceTrie;
use crate::{QueryError, TransactError};
use crate::db::key::KEY_MAX_EID;

pub(crate) struct MaxEid(pub i32);

impl MaxEid {
    pub fn take(self, count: usize) -> (Self, Vec<Ent>) {
        let Self(start) = self;
        let end = start + count as i32;
        let ids = (start..end).map(Ent).collect();
        (Self(end), ids)
    }
    pub fn update(mut self, trie: SpaceTrie, eid: i32) -> Result<SpaceTrie, TransactError> {
        if eid < self.0 {
            Ok(trie)
        } else {
            self.0 = eid + 1;
            self.write(trie)
        }
    }
    pub fn write(self, trie: SpaceTrie) -> Result<SpaceTrie, TransactError> {
        let trie = trie.insert(KEY_MAX_EID, MemValue::from(self.0 as u32))?;
        Ok(trie)
    }
    pub fn read(trie: &SpaceTrie) -> Result<Self, QueryError> {
        if let Some(MemValue::U32(value)) = trie.query_value(KEY_MAX_EID)? {
            Ok(Self(value as i32))
        } else {
            Ok(Self(0))
        }
    }
}