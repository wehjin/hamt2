use crate::trie::core::key::TrieKey;
use crate::trie::core::map_base::TrieMapBase;
use crate::trie::mem::slot::MemSlot;
use crate::trie::mem::value::MemValue;
use crate::QueryError;
use futures::stream;

impl TrieMapBase {
    pub async fn query_value(&self, key: TrieKey) -> Result<Option<MemValue>, QueryError> {
        let TrieMapBase::Mem(map, base) = self;
        let value = match map.try_base_index(key) {
            Some(base_index) => Box::pin(base[base_index].query_value(key)).await?,
            None => None,
        };
        Ok(value)
    }

    pub fn kv_stream(&self) -> impl futures::Stream<Item = (i32, MemValue)> {
        let state = State {
            jobs: Job::start(self).into_iter().collect::<Vec<_>>(),
        };
        stream::unfold(state, |mut state| async move {
            while let Some(mut job) = state.jobs.pop() {
                let TrieMapBase::Mem(_map, base) = &job.map_base;
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

    pub async fn query_keys_values(&self) -> Result<Vec<(i32, MemValue)>, QueryError> {
        let TrieMapBase::Mem(map, base) = self;
        let mut out = Vec::new();
        let slot_count = map.slot_count();
        debug_assert_eq!(slot_count, base.len());
        for base_index in 0..slot_count {
            let keys_values = Box::pin(base[base_index].query_key_values()).await?;
            out.extend(keys_values);
        }
        Ok(out)
    }
}

struct State {
    jobs: Vec<Job>,
}
struct Job {
    slot_offset: usize,
    slot_count: usize,
    map_base: TrieMapBase,
}
impl Job {
    pub fn start(map_base: &TrieMapBase) -> Option<Self> {
        let slot_count = map_base.map().slot_count();
        if slot_count == 0 {
            None
        } else {
            Some(Self {
                slot_offset: 0,
                slot_count,
                map_base: map_base.clone(),
            })
        }
    }
    pub fn next(&mut self) -> bool {
        self.slot_offset += 1;
        self.slot_offset < self.slot_count
    }
}
