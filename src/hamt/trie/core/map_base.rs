use crate::client::{QueryError, TransactError};
use crate::hamt::space;
use crate::hamt::space::core::TableRoot;
use crate::hamt::trie::core::base::TrieBase;
use crate::hamt::trie::core::key::TrieKey;
use crate::hamt::trie::core::map::TrieMap;
use crate::hamt::trie::core::value::TrieValue;
use crate::hamt::trie::mem::slot::{KvTest, MemSlot};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TrieMapBase(pub TrieMap, pub TrieBase);

impl From<&TableRoot> for TrieMapBase {
    fn from(root: &TableRoot) -> Self {
        let TableRoot(map, base) = root;
        let map = TrieMap(*map);
        let base = TrieBase::Space(*base);
        Self(map, base)
    }
}

impl TrieMapBase {
    pub fn map(&self) -> &TrieMap {
        &self.0
    }
    pub fn base(&self) -> &TrieBase {
        &self.1
    }
    pub fn empty() -> Self {
        Self(TrieMap::empty(), TrieBase::new())
    }

    pub fn one_kv(key: TrieKey, value: TrieValue) -> Result<Self, TransactError> {
        let map = TrieMap::set_key_bit(key);
        let base = TrieBase::new_kv(key, value)?;
        Ok(Self(map, base))
    }
    pub fn two_kv(
        key: TrieKey,
        value: TrieValue,
        key2: TrieKey,
        value2: TrieValue,
        reader: &impl space::Read,
    ) -> Result<Self, TransactError> {
        let one = Self::one_kv(key, value)?;
        let two = one.insert_kv(key2, value2, reader)?;
        Ok(two)
    }
    pub fn one_slot(map_index: u8, slot: MemSlot) -> Result<Self, TransactError> {
        let map = TrieMap::set_map_index_bit(map_index);
        let base = TrieBase::new_slot(slot)?;
        Ok(Self(map, base))
    }
    pub fn insert_kv(
        self,
        key: TrieKey,
        value: TrieValue,
        reader: &impl space::Read,
    ) -> Result<Self, TransactError> {
        let map_base = match self.as_slot(key, reader)? {
            Some(slot) => match slot.test_kv(&key, &value) {
                KvTest::SameValue => self,
                KvTest::ConflictOldValue => self.replace_existing_value(key, value, reader)?,
                KvTest::ConflictKeyValue => self.merge_kv(key, value, reader)?,
                KvTest::ConflictMapBase => self.merge_kv(key, value, reader)?,
            },
            None => {
                let TrieMapBase(map, base) = self;
                let base = base.insert_kv(&map, key, value, reader)?;
                let map = map.with_key(key);
                Self(map, base)
            }
        };
        Ok(map_base)
    }
    pub fn replace_existing_value(
        self,
        key: TrieKey,
        value: TrieValue,
        reader: &impl space::Read,
    ) -> Result<Self, TransactError> {
        let TrieMapBase(map, base) = self;
        let base_index = key.to_base_index(map);
        let base = base.replace_value(&map, base_index, value, reader)?;
        Ok(Self(map, base))
    }

    pub fn merge_kv(
        self,
        key: TrieKey,
        value: TrieValue,
        reader: &impl space::Read,
    ) -> Result<Self, TransactError> {
        let TrieMapBase(map, base) = self;
        let base_index = key.to_base_index(map);
        let base = base.merge_kv(&map, base_index, key, value, reader)?;
        Ok(Self(map, base))
    }

    pub fn as_slot<'a>(
        &'a self,
        key: TrieKey,
        reader: &'a impl space::Read,
    ) -> Result<Option<&'a MemSlot>, QueryError> {
        let base_index = self.0.to_base_index(key);
        if let Some(base_index) = base_index {
            let slot = self.1.as_slot(base_index, reader)?;
            Ok(Some(slot))
        } else {
            Ok(None)
        }
    }
    pub fn query_value(
        &self,
        key: TrieKey,
        reader: &impl space::Read,
    ) -> Result<Option<TrieValue>, QueryError> {
        if let Some(slot) = self.as_slot(key, reader)? {
            Ok(slot.query_value(key, reader)?)
        } else {
            Ok(None)
        }
    }
}
