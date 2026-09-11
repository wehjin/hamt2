use crate::trie::base::Base;
use crate::trie::base_storage::BaseStorageReadWrite;
use crate::trie::core::key::TrieKey;
use crate::trie::core::map_base::MapBase;
use crate::trie::mem::slot::{KvTest, MemSlot};
use crate::trie::mem::value::MemValue;
use crate::TransactError;

impl MapBase {
    pub async fn insert_kv(
        self,
        key: TrieKey,
        value: MemValue,
        storage: &mut impl BaseStorageReadWrite,
    ) -> Result<Self, TransactError> {
        let MapBase { map, base } = self;
        let post_map_base = match map.try_base_index(key) {
            Some(base_index) => {
                let read_base = storage.read(base).await.expect("read base");
                match read_base[base_index].test_kv(&key, &value) {
                    KvTest::SameValue => MapBase { map, base },
                    KvTest::ValueConflict => {
                        let post_base = Base::replace_value(read_base, base_index, value);
                        let id = storage.append(&post_base).await.expect("append base");
                        MapBase { map, base: id }
                    }
                    KvTest::KeyConflict => {
                        let post_base =
                            Base::kick_kv(read_base, base_index, key, value, storage).await;
                        let id = storage.append(&post_base).await.expect("append base");
                        MapBase { map, base: id }
                    }
                    KvTest::MapBaseConflict => {
                        let post_base = Box::pin(Base::merge_kv(
                            read_base,
                            base_index,
                            key,
                            value,
                            storage,
                        ))
                        .await?;
                        let id = storage.append(&post_base).await.expect("append base");
                        MapBase { map, base: id }
                    }
                }
            }
            None => {
                assert_eq!(false, map.is_present(key));
                let post_base = {
                    let base = storage.read(base).await.expect("read base");
                    let kv_slot = MemSlot::one_kv(key, value);
                    let kv_index = map.count_left(key);
                    base.insert_slot(kv_index, kv_slot)
                };
                let id = storage.append(&post_base).await.expect("append base");
                let post_map = map.with_key(key);
                MapBase {
                    map: post_map,
                    base: id,
                }
            }
        };
        Ok(post_map_base)
    }
}
