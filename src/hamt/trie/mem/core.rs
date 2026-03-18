use crate::client::{QueryError, TransactError};
use crate::hamt::trie::key::TrieKey;
use crate::hamt::trie::map::TrieMap;
use crate::hamt::trie::value::TrieValue;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MemMapBase {
    pub map: TrieMap,
    pub base: MemBase,
}

impl MemMapBase {
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
                let MemMapBase { map, base } = self;
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
        let MemMapBase { map, base } = self;
        let base_index = key.to_base_index(map);
        let base = base.replace_value(base_index, value)?;
        Ok(Self { map, base })
    }

    pub fn merge_kv(self, key: TrieKey, value: TrieValue) -> Result<Self, TransactError> {
        let MemMapBase { map, base } = self;
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
    pub fn query_value(&self, key: TrieKey) -> Result<Option<u32>, QueryError> {
        if let Some(slot) = self.as_slot(key)? {
            Ok(slot.query_value(key)?)
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MemBase {
    pub slots: Vec<MemSlot>,
}
impl MemBase {
    pub fn one_mb(map_base: MemMapBase) -> Result<Self, TransactError> {
        let slot = MemSlot::MapBase(map_base);
        let slots = vec![slot];
        Ok(Self { slots })
    }
    pub fn one_kv(key: TrieKey, value: TrieValue) -> Result<Self, TransactError> {
        let slot = MemSlot::one_kv(key, value)?;
        let slots = vec![slot];
        Ok(Self { slots })
    }
    pub fn insert_kv(
        self,
        base_index: usize,
        key: TrieKey,
        value: TrieValue,
    ) -> Result<Self, TransactError> {
        let slot = MemSlot::one_kv(key, value)?;
        let MemBase { mut slots } = self;
        slots.insert(base_index, slot);
        Ok(Self { slots })
    }
    pub fn replace_value(self, base_index: usize, value: TrieValue) -> Result<Self, TransactError> {
        let MemBase { mut slots } = self;
        let slot = slots.remove(base_index).replace_value(value)?;
        slots.insert(base_index, slot);
        Ok(Self { slots })
    }
    pub fn merge_kv(
        self,
        base_index: usize,
        key: TrieKey,
        value: TrieValue,
    ) -> Result<Self, TransactError> {
        let MemBase { mut slots } = self;
        let slot = slots.remove(base_index);
        let slot = slot.merge_kv(key, value)?;
        slots.insert(base_index, slot);
        Ok(Self { slots })
    }
    pub fn as_slot(&self, base_index: usize) -> Result<&MemSlot, QueryError> {
        self.slots
            .get(base_index)
            .ok_or(QueryError::BaseIndexOutOfBounds(base_index))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MemSlot {
    KeyValue(TrieKey, TrieValue),
    MapBase(MemMapBase),
}

impl MemSlot {
    pub fn one_kv(key: TrieKey, value: TrieValue) -> Result<Self, TransactError> {
        Ok(Self::KeyValue(key, value))
    }
    pub fn two_kv(
        a_key: TrieKey,
        a_value: TrieValue,
        b_key: TrieKey,
        b_value: TrieValue,
    ) -> Result<Self, TransactError> {
        let (a_map_index, b_map_index) = (a_key.map_index(), b_key.map_index());
        if a_map_index != b_map_index {
            let map_base = MemMapBase::two_kv(a_key, a_value, b_key, b_value)?;
            Ok(MemSlot::MapBase(map_base))
        } else {
            let map_index = a_map_index;
            let inner_slot = MemSlot::two_kv(a_key.next(), a_value, b_key.next(), b_value)?;
            let map_base = MemMapBase::one_slot(map_index, inner_slot)?;
            Ok(MemSlot::MapBase(map_base))
        }
    }
    pub fn replace_value(self, value: TrieValue) -> Result<Self, TransactError> {
        let MemSlot::KeyValue(key, _value) = self else {
            return Err(TransactError::InvalidSlotType);
        };
        let slot = MemSlot::KeyValue(key, value);
        Ok(slot)
    }
    pub fn merge_kv(self, key: TrieKey, value: TrieValue) -> Result<Self, TransactError> {
        match self {
            MemSlot::KeyValue(b_key, b_value) => {
                let slot = MemSlot::two_kv(b_key.next(), b_value, key.next(), value)?;
                Ok(slot)
            }
            MemSlot::MapBase(map_base) => {
                let map_base = map_base.insert_kv(key.next(), value)?;
                let slot = MemSlot::MapBase(map_base);
                Ok(slot)
            }
        }
    }
    pub fn query_value(&self, key: TrieKey) -> Result<Option<u32>, QueryError> {
        match self {
            MemSlot::KeyValue(k, v) => {
                if k.i32() != key.i32() {
                    Ok(None)
                } else {
                    let value = v.to_u32();
                    Ok(Some(value))
                }
            }
            MemSlot::MapBase(map_base) => {
                let value = map_base.query_value(key.next())?;
                Ok(value)
            }
        }
    }
    pub fn test_kv(&self, key: &TrieKey, value: &TrieValue) -> KvTest {
        match self {
            MemSlot::KeyValue(slot_key, slot_value) => {
                if key.same_i32(slot_key) {
                    if value == slot_value {
                        KvTest::SameValue
                    } else {
                        KvTest::ConflictOldValue
                    }
                } else {
                    KvTest::ConflictKeyValue
                }
            }
            MemSlot::MapBase(_) => KvTest::ConflictMapBase,
        }
    }
}

pub enum KvTest {
    SameValue,
    ConflictOldValue,
    ConflictKeyValue,
    ConflictMapBase,
}
