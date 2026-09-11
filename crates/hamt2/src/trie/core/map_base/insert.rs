use crate::TransactError;
use crate::trie::base::Base;
use crate::trie::core::key::TrieKey;
use crate::trie::core::map_base::TrieMapBase;
use crate::trie::mem::slot::{KvTest, MemSlot};
use crate::trie::mem::value::MemValue;

impl TrieMapBase {
    pub async fn insert_kv(self, key: TrieKey, value: MemValue) -> Result<Self, TransactError> {
        let TrieMapBase { map, base } = self;
        let post_map_base = match map.try_base_index(key) {
            Some(base_index) => match base[base_index].test_kv(&key, &value) {
                KvTest::SameValue => TrieMapBase { map, base },
                KvTest::ValueConflict => TrieMapBase {
                    map,
                    base: Base::replace_value(base, base_index, value),
                },
                KvTest::KeyConflict => TrieMapBase {
                    map,
                    base: Base::kick_kv(base, base_index, key, value),
                },
                KvTest::MapBaseConflict => {
                    let post_base = Box::pin(Base::merge_kv(base, base_index, key, value)).await?;
                    TrieMapBase { map, base: post_base }
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
                TrieMapBase {
                    map: post_map,
                    base: post_base,
                }
            }
        };
        Ok(post_map_base)
    }
}
