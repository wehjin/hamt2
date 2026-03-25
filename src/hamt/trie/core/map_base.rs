use crate::hamt::trie::core::key::TrieKey;
use crate::hamt::trie::core::map::TrieMap;
use crate::hamt::trie::mem::base::MemBase;
use crate::hamt::trie::mem::slot::{KvTest, MemSlot};
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::space::map_base::{SpaceKeyValue, SpaceMapBase};
use crate::space;
use crate::space::reader::SlotValue;
use crate::space::Space;
use crate::QueryError;
use crate::TransactError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TrieMapBase {
    Mem(TrieMap, MemBase),
    Space(SlotValue),
}

impl From<SlotValue> for TrieMapBase {
    fn from(slot_value: SlotValue) -> Self {
        TrieMapBase::Space(slot_value)
    }
}

impl TrieMapBase {
    pub fn map(&self) -> TrieMap {
        match self {
            TrieMapBase::Mem(map, _) => map.clone(),
            TrieMapBase::Space(slot_value) => {
                let map_base = SpaceMapBase::assert(*slot_value);
                let map = map_base.to_map();
                map
            }
        }
    }
    pub fn empty() -> Self {
        let map = TrieMap::empty();
        let base = MemBase::new();
        Self::Mem(map, base)
    }

    pub fn one_kv(key: TrieKey, value: MemValue) -> Result<Self, TransactError> {
        let map = TrieMap::set_key_bit(key);
        let base = MemBase::new_kv(key, value)?;
        Ok(Self::Mem(map, base))
    }
    pub fn two_kv(
        key: TrieKey,
        value: MemValue,
        key2: TrieKey,
        value2: MemValue,
    ) -> Result<Self, TransactError> {
        debug_assert!(key.i32() != key2.i32());
        debug_assert!(key.map_index() != key2.map_index());
        let map = TrieMap(key.to_map_bit() | key2.to_map_bit());
        let base = {
            let mut slots = Vec::new();
            if key.map_index() < key2.map_index() {
                slots.push(MemSlot::one_kv(key, value)?);
                slots.push(MemSlot::one_kv(key2, value2)?);
            } else {
                slots.push(MemSlot::one_kv(key2, value2)?);
                slots.push(MemSlot::one_kv(key, value)?);
            }
            let base = MemBase { slots };
            base
        };
        let map_base = TrieMapBase::Mem(map, base);
        Ok(map_base)
    }

    pub fn into_map_mem_base(
        self,
        reader: &impl space::Read,
    ) -> Result<(TrieMap, MemBase), QueryError> {
        match self.top_into_mem(reader)? {
            TrieMapBase::Mem(map, base) => Ok((map, base)),
            TrieMapBase::Space(_) => {
                unreachable!("Should have been converted to MemMapBase already")
            }
        }
    }

    pub fn top_into_mem(self, reader: &impl space::Read) -> Result<TrieMapBase, QueryError> {
        match self {
            TrieMapBase::Mem(_, _) => Ok(self),
            TrieMapBase::Space(slot_value) => {
                let map_base = SpaceMapBase::assert(slot_value);
                let mem_map_base = map_base.top_into_mem(reader)?;
                Ok(mem_map_base)
            }
        }
    }

    pub fn insert_kv(
        self,
        key: TrieKey,
        value: MemValue,
        reader: &impl space::Read,
    ) -> Result<Self, TransactError> {
        let pre_map_base = self.top_into_mem(reader)?;
        let pre_count = pre_map_base.query_key_values(reader)?.len();
        let post_map_base = match pre_map_base {
            TrieMapBase::Mem(_, _) => match pre_map_base.as_slot(key, reader)? {
                Some(slot) => {
                    let test = slot.test_kv(&key, &value);
                    match test {
                        KvTest::SameValue => {
                            let post = pre_map_base;
                            let post_count = post.query_key_values(reader)?.len();
                            debug_assert_eq!(post_count, pre_count);
                            post
                        }
                        KvTest::ConflictOldValue => {
                            let post = pre_map_base.replace_existing_value(key, value, reader)?;
                            let post_count = post.query_key_values(reader)?.len();
                            debug_assert_eq!(post_count, pre_count);
                            post
                        }
                        KvTest::ConflictKeyValue => {
                            let post = pre_map_base.kick_kv(key, value, reader)?;
                            let post_count = post.query_key_values(reader)?.len();
                            debug_assert_eq!(post_count, pre_count + 1);
                            post
                        }
                        KvTest::ConflictMapBase => {
                            let post = pre_map_base.merge_kv(key, value, reader)?;
                            post
                        }
                    }
                }
                None => {
                    let (pre_map, pre_base) = pre_map_base.into_map_mem_base(reader)?;
                    assert_eq!(false, pre_map.is_present(key));
                    let base_index = pre_map.count_left(key);
                    let post_base = pre_base.insert_kv(base_index, key, value)?;
                    let post_map = pre_map.with_key(key);
                    TrieMapBase::Mem(post_map, post_base)
                }
            },
            TrieMapBase::Space(_) => unreachable!(),
        };
        Ok(post_map_base)
    }

    pub fn replace_existing_value(
        self,
        key: TrieKey,
        value: MemValue,
        reader: &impl space::Read,
    ) -> Result<Self, TransactError> {
        let (map, pre_base) = self.into_map_mem_base(reader)?;
        let base_index = key.to_base_index(map);
        let post_base = MemBase::replace_value(pre_base, base_index, value)?;
        let map_base = TrieMapBase::Mem(map, post_base);
        Ok(map_base)
    }
    pub fn kick_kv(
        self,
        key: TrieKey,
        value: MemValue,
        reader: &impl space::Read,
    ) -> Result<Self, TransactError> {
        let (map, pre_base) = self.into_map_mem_base(reader)?;
        let base_index = key.to_base_index(map);
        let post_base = MemBase::kick_kv(pre_base, base_index, key, value, reader)?;
        let map_base = TrieMapBase::Mem(map, post_base);
        Ok(map_base)
    }

    pub fn merge_kv(
        self,
        key: TrieKey,
        value: MemValue,
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
            let key_values = match self {
                TrieMapBase::Mem(_, base) => {
                    debug_assert!(slot_count == base.len());
                    let slot = base.as_slot(base_index)?;
                    let keys_values = slot.query_key_values(reader)?;
                    keys_values
                }
                TrieMapBase::Space(slot_value) => {
                    let map_base = SpaceMapBase::assert(*slot_value);
                    let key_values = map_base.query_key_values(reader)?;
                    key_values
                }
            };
            result.extend(key_values);
        }
        Ok(result)
    }

    pub fn query_value(
        &self,
        key: TrieKey,
        reader: &impl space::Read,
    ) -> Result<Option<MemValue>, QueryError> {
        match self {
            TrieMapBase::Mem(_, _) => {
                if let Some(slot) = self.as_slot(key, reader)? {
                    let value = slot.query_value(key, reader)?;
                    Ok(value)
                } else {
                    Ok(None)
                }
            }
            TrieMapBase::Space(slot_value) => {
                let map_base = SpaceMapBase::assert(*slot_value);
                let value = map_base.query_value(key, reader)?;
                Ok(value)
            }
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
        _reader: &'b impl space::Read,
    ) -> Result<&'b MemSlot, QueryError> {
        let slot = match self {
            TrieMapBase::Mem(_, base) => base.as_slot(base_index)?,
            TrieMapBase::Space(_) => unreachable!(),
        };
        Ok(slot)
    }
}

impl TrieMapBase {
    pub fn into_space_map_base<T: Space>(
        self,
        extend: &mut space::Extend<T>,
    ) -> Result<SpaceMapBase, TransactError> {
        match self {
            Self::Space(slot_value) => {
                let space_map_base = SpaceMapBase::assert(slot_value);
                Ok(space_map_base)
            }
            Self::Mem(map, base) => {
                let mut slot_values: Vec<SlotValue> = vec![];
                for slot in base.slots {
                    match slot {
                        MemSlot::KeyValue(key, value) => {
                            let u32 = value.into_u32(extend)?;
                            let slot_value = SpaceKeyValue::new(key, u32).into_slot_value();
                            slot_values.push(slot_value);
                        }
                        MemSlot::MapBase(map_base) => {
                            let space_map_base = map_base.into_space_map_base(extend)?;
                            let slot_value = space_map_base.into_slot_value();
                            slot_values.push(slot_value);
                        }
                    }
                }
                let space_map_base = SpaceMapBase::new_from_slots(slot_values, map, extend)?;
                Ok(space_map_base)
            }
        }
    }
}
