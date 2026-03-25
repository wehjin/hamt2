use crate::db::key::KEY_MAX_EID;
use crate::db::Ent;
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::space::SpaceTrie;
use crate::space::Space;
use crate::{QueryError, TransactError};

pub(crate) struct MaxEid(pub i32);

impl MaxEid {
    pub fn take(self, count: usize) -> (Self, Vec<Ent>) {
        let Self(start) = self;
        let end = start + count as i32;
        let ids = (start..end).map(Ent).collect();
        (Self(end), ids)
    }
    pub fn update<T: Space>(
        mut self,
        trie: SpaceTrie<T>,
        eid: i32,
    ) -> Result<SpaceTrie<T>, TransactError> {
        if eid < self.0 {
            Ok(trie)
        } else {
            self.0 = eid + 1;
            self.write(trie)
        }
    }
    pub fn write<T: Space>(self, trie: SpaceTrie<T>) -> Result<SpaceTrie<T>, TransactError> {
        let trie = trie.insert(KEY_MAX_EID, MemValue::from(self.0 as u32))?;
        Ok(trie)
    }
    pub fn read<T: Space>(trie: &SpaceTrie<T>) -> Result<Self, QueryError> {
        if let Some(MemValue::U32(value)) = trie.query_value(KEY_MAX_EID)? {
            Ok(Self(value as i32))
        } else {
            Ok(Self(0))
        }
    }
}
