use crate::services::base;
use crate::{Base, BufferMut, HashKey, KvTest, MapBase, TrieValue};

pub async fn insert_kv<P: BufferMut>(
    map_base: MapBase,
    key: HashKey,
    value: TrieValue,
    buffer: &mut P,
) -> MapBase {
    let MapBase { map, base: base_id } = map_base;
    let slot_count = map.slot_count();
    match map.try_base_index(key) {
        Some(base_index) => {
            let read_base = Base::new(buffer.get_base(base_id, slot_count));
            match read_base[base_index].test_kv(&key, &value, buffer) {
                KvTest::SameValue => MapBase { map, base: base_id },
                KvTest::ValueConflict => {
                    let post_base = base::swap_v(read_base, base_index, value, buffer).await;
                    let id = buffer.push_base(post_base).await;
                    MapBase { map, base: id }
                }
                KvTest::KeyConflict => {
                    let post_base = base::kick_kv(read_base, base_index, key, value, buffer).await;
                    let id = buffer.push_base(post_base).await;
                    MapBase { map, base: id }
                }
                KvTest::MapBaseConflict => {
                    let post_base =
                        Box::pin(base::merge_kv(read_base, base_index, key, value, buffer)).await;
                    let id = buffer.push_base(post_base).await;
                    MapBase { map, base: id }
                }
            }
        }
        None => {
            assert_eq!(false, map.is_present(key));
            let post_slot_base = {
                let base = Base::new(buffer.get_base(base_id, slot_count));
                let kv_index = map.count_left(key);
                base::insert_kv(base, kv_index, key, value, buffer).await
            };
            let id = buffer.push_base(post_slot_base).await;
            let post_map = map.with_key(key);
            MapBase {
                map: post_map,
                base: id,
            }
        }
    }
}
