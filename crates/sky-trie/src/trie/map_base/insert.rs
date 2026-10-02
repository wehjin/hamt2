use crate::trie::{BufferMut, HashKey, KvTest, MapBase, TrieInsertError, TrieValue, base};

pub async fn insert_kv<P: BufferMut>(
    map_base: MapBase,
    key: HashKey,
    value: TrieValue,
    base_commit: &mut P,
) -> Result<MapBase, TrieInsertError> {
    let MapBase { map, base: base_id } = map_base;
    let slot_count = map.slot_count();
    let post_map_base = match map.try_base_index(key) {
        Some(base_index) => {
            let read_base = base_commit.get_base(base_id, slot_count).await;
            match read_base.as_ref()[base_index]
                .test_kv(&key, &value, base_commit)
                .await
            {
                KvTest::SameValue => MapBase { map, base: base_id },
                KvTest::ValueConflict => {
                    let post_base = base::swap_v(read_base, base_index, value, base_commit).await?;
                    let id = base_commit.push_base(post_base).await?;
                    MapBase { map, base: id }
                }
                KvTest::KeyConflict => {
                    let post_base =
                        base::kick_kv(read_base, base_index, key, value, base_commit).await?;
                    let id = base_commit.push_base(post_base).await?;
                    MapBase { map, base: id }
                }
                KvTest::MapBaseConflict => {
                    let post_base = Box::pin(base::merge_kv(
                        read_base,
                        base_index,
                        key,
                        value,
                        base_commit,
                    ))
                    .await?;
                    let id = base_commit.push_base(post_base).await?;
                    MapBase { map, base: id }
                }
            }
        }
        None => {
            assert_eq!(false, map.is_present(key));
            let post_slot_base = {
                let base = base_commit.get_base(base_id, slot_count).await;
                let kv_index = map.count_left(key);
                base::insert_kv(base, kv_index, key, value, base_commit).await?
            };
            let id = base_commit.push_base(post_slot_base).await?;
            let post_map = map.with_key(key);
            MapBase {
                map: post_map,
                base: id,
            }
        }
    };
    Ok(post_map_base)
}
