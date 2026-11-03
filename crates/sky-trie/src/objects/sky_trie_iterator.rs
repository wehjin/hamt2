use crate::{Base, MapBase, SkyTrie, Slot, TrieValue};
use std::sync::Arc;

pub struct SkyTrieIterator {
    jobs: Vec<Job>,
}

impl SkyTrieIterator {
    fn from_trie(trie: SkyTrie) -> Self {
        let jobs = Job::start(&trie.root).into_iter().collect();
        Self { jobs }
    }

    pub fn u32s(self) -> impl Iterator<Item = (i32, u32)> {
        self.filter_map(|(k, v)| match v {
            TrieValue::U32(v) => Some((k, v)),
            TrieValue::Trie(_) => None,
            TrieValue::Bytes(_) => None,
        })
    }
    pub fn tries(self) -> impl Iterator<Item = (i32, SkyTrie)> {
        self.filter_map(|(k, v)| match v {
            TrieValue::U32(_) => None,
            TrieValue::Trie(v) => Some((k, v)),
            TrieValue::Bytes(_) => None,
        })
    }
}

impl Iterator for SkyTrieIterator {
    type Item = (i32, TrieValue);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut job) = self.jobs.pop() {
            let slot = job.base.as_ref()[job.slot_offset].clone();
            match slot {
                Slot::KeyValue(key_value) => {
                    let kv = key_value.to_trie_key_trie_value();
                    if job.next() {
                        self.jobs.push(job);
                    }
                    return Some(kv);
                }
                Slot::MapBase(lower_map_base) => {
                    let lower_job = Job::start(&lower_map_base);
                    if job.next() {
                        self.jobs.push(job);
                    }
                    if let Some(lower_job) = lower_job {
                        self.jobs.push(lower_job);
                    }
                }
            }
        }
        None
    }
}

impl IntoIterator for SkyTrie {
    type Item = (i32, TrieValue);
    type IntoIter = SkyTrieIterator;

    fn into_iter(self) -> Self::IntoIter {
        SkyTrieIterator::from_trie(self)
    }
}

struct Job {
    slot_offset: usize,
    slot_count: usize,
    base: Arc<Base>,
}

impl Job {
    fn start(map_base: &MapBase) -> Option<Self> {
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

    fn next(&mut self) -> bool {
        self.slot_offset += 1;
        self.slot_offset < self.slot_count
    }
}
