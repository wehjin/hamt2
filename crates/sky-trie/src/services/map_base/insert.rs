use crate::services::base;
use crate::{HashKey, KvTest, MapBase, TrieValue};
use std::sync::Arc;

pub fn insert_kv(map_base: MapBase, key: HashKey, value: TrieValue) -> MapBase {
    let MapBase { map, base } = map_base;
    match map.try_base_index(key) {
        Some(base_index) => {
            let read_base = base.as_ref().clone();
            match read_base[base_index].test_kv(&key, &value) {
                KvTest::SameValue => MapBase { map, base },
                KvTest::ValueConflict => {
                    let post_base = base::swap_v(read_base, base_index, value);
                    MapBase {
                        map,
                        base: Arc::new(post_base),
                    }
                }
                KvTest::KeyConflict => {
                    let post_base = base::kick_kv(read_base, base_index, key, value);
                    MapBase {
                        map,
                        base: Arc::new(post_base),
                    }
                }
                KvTest::MapBaseConflict => {
                    let post_base = base::merge_kv(read_base, base_index, key, value);
                    MapBase {
                        map,
                        base: Arc::new(post_base),
                    }
                }
            }
        }
        None => {
            assert_eq!(false, map.is_present(key));
            let post_slot_base = {
                let read_base = base.as_ref().clone();
                let kv_index = map.count_left(key);
                base::insert_kv(read_base, kv_index, key, value)
            };
            let post_map = map.with_key(key);
            MapBase {
                map: post_map,
                base: Arc::new(post_slot_base),
            }
        }
    }
}
