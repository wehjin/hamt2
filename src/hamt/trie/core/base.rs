use crate::client::{QueryError, TransactError};
use crate::hamt::trie::core::key::TrieKey;
use crate::hamt::trie::core::value::TrieValue;
use crate::hamt::trie::mem::base::MemBase;
use crate::hamt::trie::mem::slot::MemSlot;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TrieBase {
    Mem(MemBase),
}

impl TrieBase {
    pub fn len(&self) -> usize {
        match self {
            Self::Mem(mem) => mem.len(),
        }
    }

    pub fn as_slot(&self, base_index: usize) -> Result<&MemSlot, QueryError> {
        let Self::Mem(mem) = self;
        mem.as_slot(base_index)
    }

    pub fn mem() -> Self {
        TrieBase::Mem(MemBase::empty())
    }

    pub fn mem_with_one_kv(key: TrieKey, value: TrieValue) -> Result<Self, TransactError> {
        Ok(TrieBase::Mem(MemBase::one_kv(key, value)?))
    }

    pub fn mem_with_one_slot(slot: MemSlot) -> Result<Self, TransactError> {
        let mem_base = MemBase { slots: vec![slot] };
        Ok(TrieBase::Mem(mem_base))
    }

    pub fn insert_kv(
        self,
        base_index: usize,
        key: TrieKey,
        value: TrieValue,
    ) -> Result<Self, TransactError> {
        let Self::Mem(mem) = self;
        let mem = MemBase::insert_kv(mem, base_index, key, value)?;
        Ok(TrieBase::Mem(mem))
    }

    pub fn replace_value(self, base_index: usize, value: TrieValue) -> Result<Self, TransactError> {
        let Self::Mem(mem) = self;
        let mem = MemBase::replace_value(mem, base_index, value)?;
        Ok(TrieBase::Mem(mem))
    }

    pub fn merge_kv(
        self,
        base_index: usize,
        key: TrieKey,
        value: TrieValue,
    ) -> Result<Self, TransactError> {
        let Self::Mem(mem) = self;
        let mem = MemBase::merge_kv(mem, base_index, key, value)?;
        Ok(TrieBase::Mem(mem))
    }
}
