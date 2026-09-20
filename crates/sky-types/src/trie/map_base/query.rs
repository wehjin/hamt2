use crate::trie::{HashKey, MapBase, Slot, TrieConfig, TrieQueryError, TrieReadPolicy, TrieValue};
use futures::Stream;
use futures::stream;

pub struct State<S: TrieReadPolicy> {
    storage: S,
    jobs: Vec<Job<<S::Config as TrieConfig>::HandleType>>,
}

pub async fn query_value<S: TrieReadPolicy>(
    map_base: &MapBase<S::Config>,
    key: HashKey,
    storage: &S,
) -> Result<Option<TrieValue<S::Config>>, TrieQueryError> {
    let MapBase { map, base: base_id } = map_base;
    let value = match map.try_base_index(key) {
        Some(base_index) => {
            let base = storage.read_base(base_id.clone()).await.expect("read base");
            Box::pin(base.as_ref()[base_index].query_value(key, storage)).await?
        }
        None => None,
    };
    Ok(value)
}

pub fn kv_stream<S: TrieReadPolicy>(
    map_base: MapBase<S::Config>,
    storage: S,
) -> impl Stream<Item = (i32, TrieValue<S::Config>)> {
    let state = State {
        storage,
        jobs: Job::start(&map_base).into_iter().collect::<Vec<_>>(),
    };
    stream::unfold(state, |mut state| async move {
        while let Some(mut job) = state.jobs.pop() {
            let base = state
                .storage
                .read_base(job.base.clone())
                .await
                .expect("read base");
            match &base.as_ref()[job.slot_offset] {
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

pub async fn query_keys_values<P: TrieReadPolicy>(
    map_base: &MapBase<P::Config>,
    storage: &P,
) -> Result<Vec<(i32, TrieValue<P::Config>)>, TrieQueryError> {
    let MapBase { map, base: base_id } = map_base;
    let mut out = Vec::new();
    let slot_count = map.slot_count();
    let base = storage.read_base(base_id.clone()).await.expect("read base");
    let base_ref = base.as_ref();
    debug_assert_eq!(slot_count, base_ref.len());
    for base_index in 0..slot_count {
        let keys_values = Box::pin(base_ref[base_index].query_key_values(storage)).await?;
        out.extend(keys_values);
    }
    Ok(out)
}

struct Job<HandleType> {
    slot_offset: usize,
    slot_count: usize,
    base: HandleType,
}
impl<HandleType: Clone> Job<HandleType> {
    pub fn start<C>(map_base: &MapBase<C>) -> Option<Self>
    where
        C: TrieConfig<HandleType = HandleType>,
    {
        let slot_count = map_base.map.slot_count();
        if slot_count == 0 {
            None
        } else {
            Some(Self {
                slot_offset: 0,
                slot_count,
                base: map_base.base.clone(),
            })
        }
    }
    pub fn next(&mut self) -> bool {
        self.slot_offset += 1;
        self.slot_offset < self.slot_count
    }
}
