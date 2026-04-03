use crate::{space, QueryError, TransactError};
use crate::trie::core::key::TrieKey;
use crate::trie::core::map_base::TrieMapBase;
use crate::trie::mem::base::MemBase;
use crate::trie::mem::slot::{KvTest, MemSlot};
use crate::trie::mem::value::MemValue;
use crate::trie::space::map_base::SpaceMapBase;

impl TrieMapBase {
    pub async fn insert_kv(
        self,
        key: TrieKey,
        value: MemValue,
        reader: &impl space::Read,
    ) -> Result<Self, TransactError> {
        let post_map_base = match self.top_into_mem(reader).await? {
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
                        let post_base =
                            MemBase::merge_kv(base, base_index, key, value, reader).await?;
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

    async fn top_into_mem(self, reader: &impl space::Read) -> Result<TrieMapBase, QueryError> {
        match self {
            TrieMapBase::Mem(_, _) => Ok(self),
            TrieMapBase::Space(slot_value) => {
                let map_base = SpaceMapBase::assert(slot_value);
                let mem_map_base = map_base.top_into_mem(reader).await?;
                Ok(mem_map_base)
            }
        }
    }
}