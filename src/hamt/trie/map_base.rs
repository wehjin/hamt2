use crate::client::{QueryError, TransactError};
use crate::hamt::trie::key::TrieKey;
use crate::hamt::trie::map::TrieMap;
use crate::hamt::trie::mem::base::MemBase;
use crate::hamt::trie::mem::slot::{KvTest, MemSlot};
use crate::hamt::trie::value::TrieValue;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TrieMapBase {
    pub map: TrieMap,
    pub base: MemBase,
}

impl TrieMapBase {
    pub fn empty() -> Self {
        Self {
            map: TrieMap::empty(),
            base: MemBase::empty(),
        }
    }

    pub fn one_kv(key: TrieKey, value: TrieValue) -> Result<Self, TransactError> {
        let map = TrieMap::set_key_bit(key);
        let base = MemBase::one_kv(key, value)?;
        Ok(Self { map, base })
    }
    pub fn two_kv(
        key: TrieKey,
        value: TrieValue,
        key2: TrieKey,
        value2: TrieValue,
    ) -> Result<Self, TransactError> {
        let one = Self::one_kv(key, value)?;
        let two = one.insert_kv(key2, value2)?;
        Ok(two)
    }
    pub fn one_slot(map_index: u8, slot: MemSlot) -> Result<Self, TransactError> {
        let map = TrieMap::set_map_index_bit(map_index);
        let base = MemBase { slots: vec![slot] };
        Ok(Self { map, base })
    }
    pub fn insert_kv(self, key: TrieKey, value: TrieValue) -> Result<Self, TransactError> {
        let map_base = match self.as_slot(key)? {
            Some(slot) => match slot.test_kv(&key, &value) {
                KvTest::SameValue => self,
                KvTest::ConflictOldValue => self.replace_existing_value(key, value)?,
                KvTest::ConflictKeyValue => self.merge_kv(key, value)?,
                KvTest::ConflictMapBase => self.merge_kv(key, value)?,
            },
            None => {
                let TrieMapBase { map, base } = self;
                let map = map.with_key(key);
                let Some(base_index) = map.to_base_index(key) else {
                    return Err(TransactError::SlotEmpty);
                };
                let base = base.insert_kv(base_index, key, value)?;
                Self { map, base }
            }
        };
        Ok(map_base)
    }
    pub fn replace_existing_value(
        self,
        key: TrieKey,
        value: TrieValue,
    ) -> Result<Self, TransactError> {
        let TrieMapBase { map, base } = self;
        let base_index = key.to_base_index(map);
        let base = base.replace_value(base_index, value)?;
        Ok(Self { map, base })
    }

    pub fn merge_kv(self, key: TrieKey, value: TrieValue) -> Result<Self, TransactError> {
        let TrieMapBase { map, base } = self;
        let base_index = key.to_base_index(map);
        let base = base.merge_kv(base_index, key, value)?;
        Ok(Self { map, base })
    }

    pub fn as_slot(&self, key: TrieKey) -> Result<Option<&MemSlot>, QueryError> {
        let base_index = self.map.to_base_index(key);
        if let Some(base_index) = base_index {
            let slot = self.base.as_slot(base_index)?;
            Ok(Some(slot))
        } else {
            Ok(None)
        }
    }
    pub fn query_value(&self, key: TrieKey) -> Result<Option<TrieValue>, QueryError> {
        if let Some(slot) = self.as_slot(key)? {
            Ok(slot.query_value(key)?)
        } else {
            Ok(None)
        }
    }
}
