use crate::trie::map_base::{query_keys_values, query_value, two_kv};
use crate::trie::{
    HashKey, MapBase, SlotBase, SlotMap, TrieConfig, TrieQueryError, TrieReadPolicy, TrieValue,
    TrieWritePolicy,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "C::HandleType: Serialize",
    deserialize = "C::HandleType: Deserialize<'de>"
))]
pub enum Slot<C: TrieConfig> {
    KeyValue(i32, TrieValue<C>),
    MapBase(MapBase<C>),
}

impl<C: TrieConfig> Slot<C> {
    pub fn one_kv(key: HashKey, value: TrieValue<C>) -> Self {
        Self::KeyValue(key.i32(), value)
    }
    pub async fn two_kv<P: TrieWritePolicy<Config = C>>(
        a_key: HashKey,
        a_value: TrieValue<C>,
        b_key: HashKey,
        b_value: TrieValue<C>,
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
            let base = SlotBase { slots: vec![slot] };
            let id = policy.commit_base(base).await.expect("append base");
            Slot::MapBase(MapBase { map, base: id })
        } else {
            let map_base = two_kv(a_key, a_value, b_key, b_value, policy).await;
            Slot::MapBase(map_base)
        }
    }
    pub fn replace_value(self, value: TrieValue<C>) -> Self {
        let Slot::KeyValue(key, _value) = self else {
            unreachable!("Should be a key-value slot, not a map-base slot:")
        };
        Slot::KeyValue(key, value)
    }
    pub async fn query_key_values<P: TrieReadPolicy<Config = C>>(
        &self,
        storage: &P,
    ) -> Result<Vec<(i32, TrieValue<C>)>, TrieQueryError> {
        match self {
            Slot::KeyValue(key, value) => Ok(vec![(*key, value.clone())]),
            Slot::MapBase(map_base) => query_keys_values(map_base, storage).await,
        }
    }
    pub async fn query_value<P: TrieReadPolicy<Config = C>>(
        &self,
        key: HashKey,
        storage: &P,
    ) -> Result<Option<TrieValue<C>>, TrieQueryError> {
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
    pub fn test_kv(&self, key: &HashKey, value: &TrieValue<C>) -> KvTest {
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
