use crate::space;
use crate::space::core::reader::SlotValue;
use crate::space::Space;
use crate::trie::core::key::TrieKey;
use crate::trie::core::map::TrieMap;
use crate::trie::mem::base::MemBase;
use crate::trie::mem::slot::{KvTest, MemSlot};
use crate::trie::mem::value::MemValue;
use crate::trie::space::map_base::SpaceMapBase;
use crate::QueryError;
use crate::TransactError;
use serde::{Deserialize, Serialize};

pub mod query;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TrieMapBase {
    Mem(TrieMap, MemBase),
    Space(SlotValue),
}

impl TrieMapBase {
    pub fn from_slot_value(slot_value: SlotValue) -> Self {
        let space_map_base = SpaceMapBase::assert(slot_value);
        Self::Space(space_map_base.into_slot_value())
    }

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

    pub fn one_kv(key: TrieKey, value: MemValue) -> Self {
        let map = TrieMap::set_key_bit(key);
        let base = MemBase::new_kv(key, value);
        Self::Mem(map, base)
    }
    pub fn two_kv(key: TrieKey, value: MemValue, key2: TrieKey, value2: MemValue) -> Self {
        debug_assert!(key.i32() != key2.i32());
        debug_assert!(key.map_index() != key2.map_index());
        let map = TrieMap(key.to_map_bit() | key2.to_map_bit());
        let base = {
            let mut slots = Vec::new();
            if key.map_index() < key2.map_index() {
                slots.push(MemSlot::one_kv(key, value));
                slots.push(MemSlot::one_kv(key2, value2));
            } else {
                slots.push(MemSlot::one_kv(key2, value2));
                slots.push(MemSlot::one_kv(key, value));
            }
            let base = MemBase { slots };
            base
        };
        TrieMapBase::Mem(map, base)
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
        let post_map_base = match self.top_into_mem(reader)? {
            TrieMapBase::Mem(map, base) => match map.try_base_index(key) {
                Some(base_index) => match base[base_index].test_kv(&key, &value) {
                    KvTest::SameValue => TrieMapBase::Mem(map, base),
                    KvTest::ValueConflict => {
                        TrieMapBase::Mem(map, MemBase::replace_value(base, base_index, value))
                    }
                    KvTest::KeyConflict => {
                        TrieMapBase::Mem(map, MemBase::kick_kv(base, base_index, key, value))
                    }
                    KvTest::MapBaseConflict => {
                        let post_base = MemBase::merge_kv(base, base_index, key, value, reader)?;
                        TrieMapBase::Mem(map, post_base)
                    }
                },
                None => {
                    assert_eq!(false, map.is_present(key));
                    let post_base = {
                        let kv_slot = MemSlot::one_kv(key, value);
                        let kv_index = map.count_left(key);
                        base.insert_slot(kv_index, kv_slot)
                    };
                    let post_map = map.with_key(key);
                    TrieMapBase::Mem(post_map, post_base)
                }
            },
            TrieMapBase::Space(_) => {
                unreachable!("Should have been converted to MemMapBase already")
            }
        };
        Ok(post_map_base)
    }
}

impl TrieMapBase {
    pub fn query_value(
        &self,
        key: TrieKey,
        reader: &impl space::Read,
    ) -> Result<Option<MemValue>, QueryError> {
        let value = match self {
            TrieMapBase::Mem(map, base) => match map.try_base_index(key) {
                Some(base_index) => base[base_index].query_value(key, reader)?,
                None => None,
            },
            TrieMapBase::Space(slot_value) => {
                SpaceMapBase::assert(*slot_value).query_value(key, reader)?
            }
        };
        Ok(value)
    }
}

impl TrieMapBase {
    pub fn into_space_map_base<T: Space>(
        self,
        extend: &mut space::Extend<T>,
    ) -> Result<SpaceMapBase, TransactError> {
        let space_map_base = match self {
            Self::Space(slot_value) => SpaceMapBase::assert(slot_value),
            Self::Mem(map, base) => SpaceMapBase::save(extend, map, base)?,
        };
        Ok(space_map_base)
    }
}
