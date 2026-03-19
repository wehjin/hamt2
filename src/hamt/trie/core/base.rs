use crate::client::{QueryError, TransactError};
use crate::hamt::space;
use crate::hamt::trie::core::key::TrieKey;
use crate::hamt::trie::core::value::TrieValue;
use crate::hamt::trie::mem::base::MemBase;
use crate::hamt::trie::mem::slot::MemSlot;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TrieBase {
    Mem(MemBase),
    Space(space::TableAddr),
}

impl TrieBase {
    pub fn len(&self) -> usize {
        match self {
            Self::Mem(mem) => mem.len(),
            TrieBase::Space(_) => {
                unimplemented!();
            }
        }
    }

    pub fn as_slot(&self, base_index: usize) -> Result<&MemSlot, QueryError> {
        match self {
            TrieBase::Mem(mem) => mem.as_slot(base_index),
            TrieBase::Space(_) => {
                unimplemented!();
            }
        }
    }

    pub fn new() -> Self {
        TrieBase::Mem(MemBase::new())
    }

    pub fn new_kv(key: TrieKey, value: TrieValue) -> Result<Self, TransactError> {
        Ok(TrieBase::Mem(MemBase::new_kv(key, value)?))
    }

    pub fn new_slot(slot: MemSlot) -> Result<Self, TransactError> {
        let mem_base = MemBase { slots: vec![slot] };
        Ok(TrieBase::Mem(mem_base))
    }

    pub fn insert_kv(
        self,
        base_index: usize,
        key: TrieKey,
        value: TrieValue,
    ) -> Result<Self, TransactError> {
        match self {
            TrieBase::Space(_) => unimplemented!(),
            TrieBase::Mem(mem) => {
                let base = MemBase::insert_kv(mem, base_index, key, value)?;
                Ok(TrieBase::Mem(base))
            }
        }
    }

    pub fn replace_value(self, base_index: usize, value: TrieValue) -> Result<Self, TransactError> {
        match self {
            TrieBase::Space(_) => unimplemented!(),
            TrieBase::Mem(base) => {
                let base = MemBase::replace_value(base, base_index, value)?;
                Ok(TrieBase::Mem(base))
            }
        }
    }

    pub fn merge_kv(
        self,
        base_index: usize,
        key: TrieKey,
        value: TrieValue,
    ) -> Result<Self, TransactError> {
        match self {
            TrieBase::Space(_) => unimplemented!(),
            TrieBase::Mem(base) => {
                let base = MemBase::merge_kv(base, base_index, key, value)?;
                Ok(TrieBase::Mem(base))
            }
        }
    }
}
