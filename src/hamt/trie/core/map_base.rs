use crate::hamt::trie::core::key::TrieKey;
use crate::hamt::trie::core::map::TrieMap;
use crate::hamt::trie::core::value::TrieValue;
use crate::hamt::trie::mem::base::MemBase;
use crate::hamt::trie::mem::slot::{KvTest, MemSlot};
use crate::hamt::trie::mem::value::MemValue;
use crate::space;
use crate::space::table::TableRoot;
use crate::space::{TableAddr, Value};
use crate::QueryError;
use crate::TransactError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum TrieMapBase {
    Mem(TrieMap, MemBase),
    Space(TrieMap, TableAddr),
}

impl From<&TableRoot> for TrieMapBase {
    fn from(root: &TableRoot) -> Self {
        let TableRoot(map, base) = root;
        let map = TrieMap(*map);
        Self::Space(map, *base)
    }
}

impl TrieMapBase {
    pub fn map(&self) -> &TrieMap {
        match self {
            TrieMapBase::Mem(map, _) => &map,
            TrieMapBase::Space(map, _) => &map,
        }
    }
    pub fn empty() -> Self {
        let map = TrieMap::empty();
        let base = MemBase::new();
        Self::Mem(map, base)
    }

    pub fn one_kv(key: TrieKey, value: TrieValue) -> Result<Self, TransactError> {
        let map = TrieMap::set_key_bit(key);
        let base = MemBase::new_kv(key, value)?;
        Ok(Self::Mem(map, base))
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
        let base = MemBase { slots: vec![slot] };
        Ok(TrieMapBase::Mem(map, base))
    }

    pub fn into_map_mem_base(
        self,
        reader: &impl space::Read,
    ) -> Result<(TrieMap, MemBase), QueryError> {
        match self {
            TrieMapBase::Mem(map, base) => Ok((map, base)),
            TrieMapBase::Space(map, base_addr) => {
                let base = MemBase::load(&base_addr, map.slot_count(), reader)?;
                Ok((map, base))
            }
        }
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
                let (pre_map, pre_base) = self.into_map_mem_base(reader)?;
                assert_eq!(false, pre_map.is_present(key));
                let base_index = pre_map.count_left(key);
                let post_base = pre_base.insert_kv(base_index, key, value)?;
                let post_map = pre_map.with_key(key);
                TrieMapBase::Mem(post_map, post_base)
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
        let (map, pre_base) = self.into_map_mem_base(reader)?;
        let base_index = key.to_base_index(map);
        let post_base = MemBase::replace_value(pre_base, base_index, value)?;
        let map_base = TrieMapBase::Mem(map, post_base);
        Ok(map_base)
    }

    pub fn merge_kv(
        self,
        key: TrieKey,
        value: TrieValue,
        reader: &impl space::Read,
    ) -> Result<Self, TransactError> {
        let (map, pre_base) = self.into_map_mem_base(reader)?;
        let base_index = key.to_base_index(map);
        let post_base = MemBase::merge_kv(pre_base, base_index, key, value, reader)?;
        let map_base = TrieMapBase::Mem(map, post_base);
        Ok(map_base)
    }
}

impl TrieMapBase {
    pub fn query_key_values(
        &self,
        reader: &impl space::Read,
    ) -> Result<Vec<(i32, MemValue)>, QueryError> {
        let mut result = Vec::new();
        let slot_count = self.map().slot_count();
        for base_index in 0..slot_count {
            let slot = match self {
                TrieMapBase::Mem(_, base) => base.as_slot(base_index)?,
                TrieMapBase::Space(_, base_addr) => reader.read_slot(base_addr, base_index)?,
            };
            let keys_values = slot.query_key_values(reader)?;
            result.extend(keys_values);
        }
        Ok(result)
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

    pub fn as_slot<'a>(
        &'a self,
        key: TrieKey,
        reader: &'a impl space::Read,
    ) -> Result<Option<&'a MemSlot>, QueryError> {
        let base_index = self.map().to_base_index(key);
        let slot = if let Some(base_index) = base_index {
            let slot = self.as_slot_base_index(base_index, reader)?;
            Some(slot)
        } else {
            None
        };
        Ok(slot)
    }

    fn as_slot_base_index<'b>(
        &'b self,
        base_index: usize,
        reader: &'b impl space::Read,
    ) -> Result<&'b MemSlot, QueryError> {
        let slot = match self {
            TrieMapBase::Mem(_, base) => base.as_slot(base_index)?,
            TrieMapBase::Space(_, base_addr) => reader.read_slot(base_addr, base_index)?,
        };
        Ok(slot)
    }
}

impl TrieMapBase {
    pub fn into_map_base_addr(
        self,
        extend: &mut space::Extend,
    ) -> Result<(TrieMap, TableAddr), TransactError> {
        match self {
            Self::Space(map, addr) => Ok((map, addr)),
            Self::Mem(map, base) => {
                let mut items = vec![];
                for slot in base.slots {
                    match slot {
                        MemSlot::KeyValue(key, value) => {
                            let value_addr = match value {
                                TrieValue::Space(value_addr) => value_addr,
                                TrieValue::Mem(value) => {
                                    let space_value = match value {
                                        MemValue::U32(value) => Value::U32(value),
                                        MemValue::MapBase(map_base) => {
                                            let (map, base_addr) =
                                                map_base.into_map_base_addr(extend)?;
                                            Value::MapBase(map.0, base_addr)
                                        }
                                    };
                                    extend.add_value(space_value)
                                }
                            };
                            let item = MemSlot::KeyValue(key, TrieValue::Space(value_addr));
                            items.push(item);
                        }
                        MemSlot::MapBase(map_base) => {
                            let (map, base_addr) = map_base.into_map_base_addr(extend)?;
                            let space_map_base = Self::Space(map, base_addr);
                            let item = MemSlot::MapBase(space_map_base);
                            items.push(item);
                        }
                    }
                }
                let base_addr = extend.add_items(items);
                Ok((map, base_addr))
            }
        }
    }
}
