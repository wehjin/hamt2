use crate::services::map_base::{query_keys_values, query_value, two_kv};
use crate::types::key_value::KeyValue;
use crate::{Base, HashKey, MapBase, SlotMap, TrieValue};
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Slot {
    KeyValue(KeyValue),
    MapBase(MapBase),
}

impl Slot {
    pub fn one_kv(key: HashKey, value: TrieValue) -> Self {
        let key_value = KeyValue::from_hash_key_trie_value(key, value);
        Slot::KeyValue(key_value)
    }
    pub fn two_kv(
        a_key: HashKey,
        a_value: TrieValue,
        b_key: HashKey,
        b_value: TrieValue,
    ) -> Self {
        let (a_map_index, b_map_index) = (a_key.map_index(), b_key.map_index());
        if a_map_index == b_map_index {
            let map = SlotMap::set_map_index_bit(a_map_index);
            let slot = Slot::two_kv(a_key.next(), a_value, b_key.next(), b_value);
            let base = Arc::new(Base { slots: vec![slot] });
            Slot::MapBase(MapBase { map, base })
        } else {
            let map_base = two_kv(a_key, a_value, b_key, b_value);
            Slot::MapBase(map_base)
        }
    }
    pub fn replace_value(self, value: TrieValue) -> Self {
        let Slot::KeyValue(key_value) = self else {
            unreachable!("Should be a key-value slot, not a map-base slot:")
        };
        let key_value = key_value.replace_value(value);
        Slot::KeyValue(key_value)
    }
    pub fn query_key_values(&self) -> Vec<(i32, TrieValue)> {
        match self {
            Slot::KeyValue(key_value) => {
                let key_value = key_value.to_trie_key_trie_value();
                vec![key_value]
            }
            Slot::MapBase(map_base) => query_keys_values(map_base.clone()),
        }
    }
    pub fn query_value(&self, key: HashKey) -> Option<TrieValue> {
        match self {
            Slot::KeyValue(kv) => {
                let (k, v) = kv.to_trie_key_trie_value();
                if k != key.i32() { None } else { Some(v) }
            }
            Slot::MapBase(map_base) => query_value(map_base.clone(), key.next()),
        }
    }
    pub fn test_kv(&self, key: &HashKey, value: &TrieValue) -> KvTest {
        match self {
            Slot::KeyValue(key_value) => {
                let (slot_key, slot_value) = key_value.to_trie_key_trie_value();
                if key.i32() == slot_key {
                    if value == &slot_value {
                        KvTest::SameValue
                    } else {
                        KvTest::ValueConflict
                    }
                } else {
                    KvTest::KeyConflict
                }
            }
            Slot::MapBase(_) => KvTest::MapBaseConflict,
        }
    }
}

pub enum KvTest {
    SameValue,
    ValueConflict,
    KeyConflict,
    MapBaseConflict,
}
