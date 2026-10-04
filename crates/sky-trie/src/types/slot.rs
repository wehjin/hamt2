use crate::services::map_base::{query_keys_values, query_value, two_kv};
use crate::types::key_value::KeyValue;
use crate::{Base, Buffer, BufferMut, ByteData, HashKey, MapBase, SlotMap, TrieValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Slot {
    KeyValue(KeyValue),
    MapBase(MapBase),
    ByteData(ByteData),
}

impl Slot {
    pub async fn one_kv(key: HashKey, value: TrieValue, buffer: &mut impl BufferMut) -> Self {
        let key_value = KeyValue::from_hash_key_trie_value(key, value, buffer).await;
        Slot::KeyValue(key_value)
    }
    pub async fn two_kv<P: BufferMut>(
        a_key: HashKey,
        a_value: TrieValue,
        b_key: HashKey,
        b_value: TrieValue,
        policy: &mut P,
    ) -> Self {
        let (a_map_index, b_map_index) = (a_key.map_index(), b_key.map_index());
        if a_map_index == b_map_index {
            let map = SlotMap::set_map_index_bit(a_map_index);
            let slot = Box::pin(Slot::two_kv(
                a_key.next(),
                a_value,
                b_key.next(),
                b_value,
                policy,
            ))
            .await;
            let base = Base { slots: vec![slot] };
            let id = policy.push_base(base).await;
            Slot::MapBase(MapBase { map, base: id })
        } else {
            let map_base = two_kv(a_key, a_value, b_key, b_value, policy).await;
            Slot::MapBase(map_base)
        }
    }
    pub async fn replace_value(self, value: TrieValue, buffer: &mut impl BufferMut) -> Self {
        let Slot::KeyValue(key_value) = self else {
            unreachable!("Should be a key-value slot, not a map-base slot:")
        };
        let key_value = key_value.replace_value(value, buffer).await;
        Slot::KeyValue(key_value)
    }
    pub async fn query_key_values<P: Buffer>(&self, storage: &P) -> Vec<(i32, TrieValue)> {
        match self {
            Slot::KeyValue(key_value) => {
                let key_value = key_value.to_trie_key_trie_value(storage);
                vec![key_value]
            }
            Slot::MapBase(map_base) => query_keys_values(*map_base, storage).await,
            Slot::ByteData(_) => unreachable!("byte-data should not appear at this level"),
        }
    }
    pub async fn query_value<P: Buffer>(&self, key: HashKey, storage: &P) -> Option<TrieValue> {
        match self {
            Slot::KeyValue(kv) => {
                let (k, v) = kv.to_trie_key_trie_value(storage);
                if k != key.i32() { None } else { Some(v) }
            }
            Slot::MapBase(map_base) => query_value(*map_base, key.next(), storage).await,
            Slot::ByteData(_) => unreachable!("byte-data should not appear at this level"),
        }
    }
    pub fn test_kv(&self, key: &HashKey, value: &TrieValue, buffer: &impl Buffer) -> KvTest {
        match self {
            Slot::KeyValue(key_value) => {
                let (slot_key, slot_value) = key_value.to_trie_key_trie_value(buffer);
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
            Slot::ByteData(_) => unreachable!("byte-data should not appear at this level"),
        }
    }
}

pub enum KvTest {
    SameValue,
    ValueConflict,
    KeyConflict,
    MapBaseConflict,
}
