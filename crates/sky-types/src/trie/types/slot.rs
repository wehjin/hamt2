use crate::trie::map_base::{query_keys_values, query_value, two_kv};
use crate::trie::{
    HashKey, MapBase, SlotBase, SlotMap, TrieQueryError, TrieReadPolicy, TrieValue, TrieWritePolicy,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Slot<HandleType> {
    KeyValue(i32, TrieValue<HandleType>),
    MapBase(MapBase<HandleType>),
}

impl<HandleType: Clone + Eq + PartialEq> Slot<HandleType> {
    pub fn one_kv(key: HashKey, value: TrieValue<HandleType>) -> Self {
        Self::KeyValue(key.i32(), value)
    }
    pub async fn two_kv<P: TrieWritePolicy<HandleType = HandleType>>(
        a_key: HashKey,
        a_value: TrieValue<HandleType>,
        b_key: HashKey,
        b_value: TrieValue<HandleType>,
        policy: &mut P,
    ) -> Self {
        debug_assert!(a_key.i32() != b_key.i32());
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
            let base = P::BaseType::from(SlotBase { slots: vec![slot] });
            let id = policy.commit_base(base).await.expect("append base");
            Slot::MapBase(MapBase { map, base: id })
        } else {
            let map_base = two_kv(a_key, a_value, b_key, b_value, policy).await;
            Slot::MapBase(map_base)
        }
    }
    pub fn replace_value(self, value: TrieValue<HandleType>) -> Self {
        let Slot::KeyValue(key, _value) = self else {
            unreachable!("Should be a key-value slot, not a map-base slot:")
        };
        Slot::KeyValue(key, value)
    }
    pub async fn query_key_values<P: TrieReadPolicy<HandleType = HandleType>>(
        &self,
        storage: &P,
    ) -> Result<Vec<(i32, TrieValue<HandleType>)>, TrieQueryError> {
        match self {
            Slot::KeyValue(key, value) => Ok(vec![(*key, value.clone())]),
            Slot::MapBase(map_base) => query_keys_values(map_base, storage).await,
        }
    }
    pub async fn query_value<P: TrieReadPolicy<HandleType = HandleType>>(
        &self,
        key: HashKey,
        storage: &P,
    ) -> Result<Option<TrieValue<HandleType>>, TrieQueryError> {
        match self {
            Slot::KeyValue(k, v) => {
                if *k != key.i32() {
                    Ok(None)
                } else {
                    Ok(Some(v.clone()))
                }
            }
            Slot::MapBase(map_base) => query_value(map_base, key.next(), storage).await,
        }
    }
    pub fn test_kv(&self, key: &HashKey, value: &TrieValue<HandleType>) -> KvTest {
        match self {
            Slot::KeyValue(slot_key, slot_value) => {
                if key.i32() == *slot_key {
                    if value == slot_value {
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
