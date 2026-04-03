use crate::space::Read;
use crate::trie::core::key::TrieKey;
use crate::trie::core::map_base::TrieMapBase;
use crate::trie::core::query::QueryKeysValues;
use crate::trie::mem::slot::MemSlot;
use crate::trie::mem::value::MemValue;
use crate::trie::space::map_base::SpaceMapBase;
use crate::{space, QueryError};
use futures::stream;

impl TrieMapBase {
    pub async fn query_value(
        &self,
        key: TrieKey,
        reader: &impl space::Read,
    ) -> Result<Option<MemValue>, QueryError> {
        let value = match self {
            TrieMapBase::Mem(map, base) => match map.try_base_index(key) {
                Some(base_index) => Box::pin(base[base_index].query_value(key, reader)).await?,
                None => None,
            },
            TrieMapBase::Space(slot_value) => {
                SpaceMapBase::assert(*slot_value)
                    .query_value(key, reader)
                    .await?
            }
        };
        Ok(value)
    }

    pub fn kv_stream<R: Read + Clone>(
        &self,
        reader: &R,
    ) -> impl futures::Stream<Item = (i32, MemValue)> {
        let state = State {
            reader: reader.clone(),
            jobs: Job::start(self).into_iter().collect::<Vec<_>>(),
        };
        stream::unfold(state, |mut state| async move {
            while let Some(mut job) = state.jobs.pop() {
                match &job.map_base {
                    TrieMapBase::Mem(_map, base) => match &base[job.slot_offset] {
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
                    },
                    TrieMapBase::Space(slot_value) => {
                        // We will transform this space map-base into a mem map-base
                        // and restart the job. Slot_offset should always be 0 here.
                        debug_assert_eq!(job.slot_offset, 0);
                        let space_map_base = SpaceMapBase::assert(*slot_value);
                        match space_map_base.top_into_mem(&state.reader).await {
                            Ok(mem_map_base) => {
                                if let Some(mem_job) = Job::start(&mem_map_base) {
                                    // We have a mem job that matches the current space job,
                                    // so put it back in the queue and continue processing the
                                    // queue.
                                    state.jobs.push(mem_job);
                                } else {
                                    // The mem map-base is empty. This job is already done,
                                    // so drop the current job and continue processing the
                                    // queue.
                                }
                            }
                            Err(e) => panic!("Read failed: {:?}", e),
                        }
                    }
                }
            }
            None
        })
    }
}

struct State<R: space::Read> {
    reader: R,
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

impl QueryKeysValues for TrieMapBase {
    async fn query_keys_values(
        &self,
        reader: &impl Read,
    ) -> Result<Vec<(i32, MemValue)>, QueryError> {
        let mut out = Vec::new();
        match self {
            TrieMapBase::Mem(map, base) => {
                let slot_count = map.slot_count();
                debug_assert_eq!(slot_count, base.len());
                for base_index in 0..slot_count {
                    let keys_values = Box::pin(base[base_index].query_key_values(reader)).await?;
                    out.extend(keys_values);
                }
            }
            TrieMapBase::Space(slot_value) => {
                let map_base = SpaceMapBase::assert(*slot_value);
                out.extend(map_base.query_keys_values(reader).await?);
            }
        }
        Ok(out)
    }
}
