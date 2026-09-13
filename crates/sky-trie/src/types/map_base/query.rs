use crate::TrieQueryError;
use crate::trie_storage::ReadTrieStorage;
use crate::types::HashKey;
use crate::types::map_base::MapBase;
use crate::types::slot::Slot;
use crate::types::trie_value::TrieValue;
use futures::Stream;
use futures::stream;

pub struct State<'a, S: ReadTrieStorage> {
    storage: &'a S,
    jobs: Vec<Job>,
}

impl MapBase {
    pub async fn query_value(
        &self,
        key: HashKey,
        storage: &impl ReadTrieStorage,
    ) -> Result<Option<TrieValue>, TrieQueryError> {
        let MapBase { map, base } = self;
        let value = match map.try_base_index(key) {
            Some(base_index) => {
                let base = storage.read(*base).await.expect("read base");
                Box::pin(base[base_index].query_value(key, storage)).await?
            }
            None => None,
        };
        Ok(value)
    }

    pub fn kv_stream<'a, S: ReadTrieStorage>(
        self,
        storage: &'a S,
    ) -> impl Stream<Item = (i32, TrieValue)> + 'a {
        let state = State {
            storage,
            jobs: Job::start(&self).into_iter().collect::<Vec<_>>(),
        };
        stream::unfold(state, |mut state| async move {
            while let Some(mut job) = state.jobs.pop() {
                let base = state.storage.read(job.base).await.expect("read base");
                match &base[job.slot_offset] {
                    Slot::KeyValue(key, value) => {
                        // Found a key and value. We finish by moving the current
                        // job forward and yielding the key-value pair.
                        let kv = (*key, value.clone());
                        if job.next() {
                            state.jobs.push(job);
                        }
                        return Some((kv, state));
                    }
                    Slot::MapBase(lower_map_base) => {
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
        storage: &impl ReadTrieStorage,
    ) -> Result<Vec<(i32, TrieValue)>, TrieQueryError> {
        let MapBase { map, base } = self;
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
    base: crate::types::slot_base_id::SlotBaseId,
}
impl Job {
    pub fn start(map_base: &MapBase) -> Option<Self> {
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
