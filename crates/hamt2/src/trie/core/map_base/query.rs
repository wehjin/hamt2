use crate::trie::base_storage::BaseStorageRead;
use crate::trie::core::key::TrieKey;
use crate::trie::core::map_base::TrieMapBase;
use crate::trie::mem::slot::MemSlot;
use crate::trie::mem::value::MemValue;
use crate::QueryError;
use futures::stream;
use futures::Stream;

pub struct State<'a, S: BaseStorageRead> {
    storage: &'a S,
    jobs: Vec<Job>,
}

impl TrieMapBase {
    pub async fn query_value(
        &self,
        key: TrieKey,
        storage: &impl BaseStorageRead,
    ) -> Result<Option<MemValue>, QueryError> {
        let TrieMapBase { map, base } = self;
        let value = match map.try_base_index(key) {
            Some(base_index) => {
                let base = storage.read(*base).await.expect("read base");
                Box::pin(base[base_index].query_value(key, storage)).await?
            }
            None => None,
        };
        Ok(value)
    }

    pub fn kv_stream<'a, S: BaseStorageRead + 'a>(
        &'a self,
        storage: &'a S,
    ) -> impl Stream<Item = (i32, MemValue)> + 'a {
        let state = State {
            storage,
            jobs: Job::start(self).into_iter().collect::<Vec<_>>(),
        };
        stream::unfold(state, |mut state| async move {
            while let Some(mut job) = state.jobs.pop() {
                let base = state.storage.read(job.base).await.expect("read base");
                match &base[job.slot_offset] {
                    MemSlot::KeyValue(key, value) => {
                        // Found a key and value. We finish by moving the current
                        // job forward and yielding the key-value pair.
                        let kv = (*key, value.clone());
                        if job.next() {
                            state.jobs.push(job);
                        }
                        return Some((kv, state));
                    }
                    MemSlot::MapBase(lower_map_base) => {
                        // Found a lower map-base. We will move the current job
                        // forward and start a new job for the lower map-base.
                        let lower_job = Job::start(lower_map_base);
                        if job.next() {
                            state.jobs.push(job);
                        }
                        if let Some(new_job) = lower_job {
                            state.jobs.push(new_job);
                        }
                    }
                }
            }
            None
        })
    }

    pub async fn query_keys_values(
        &self,
        storage: &impl BaseStorageRead,
    ) -> Result<Vec<(i32, MemValue)>, QueryError> {
        let TrieMapBase { map, base } = self;
        let mut out = Vec::new();
        let slot_count = map.slot_count();
        let base = storage.read(*base).await.expect("read base");
        debug_assert_eq!(slot_count, base.len());
        for base_index in 0..slot_count {
            let keys_values = Box::pin(base[base_index].query_key_values(storage)).await?;
            out.extend(keys_values);
        }
        Ok(out)
    }
}

struct Job {
    slot_offset: usize,
    slot_count: usize,
    base: crate::trie::base::BaseId,
}
impl Job {
    pub fn start(map_base: &TrieMapBase) -> Option<Self> {
        let slot_count = map_base.map.slot_count();
        if slot_count == 0 {
            None
        } else {
            Some(Self {
                slot_offset: 0,
                slot_count,
                base: map_base.base,
            })
        }
    }
    pub fn next(&mut self) -> bool {
        self.slot_offset += 1;
        self.slot_offset < self.slot_count
    }
}
