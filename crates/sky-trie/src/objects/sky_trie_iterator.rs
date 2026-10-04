use crate::{Buffer, BufferIndex, MapBase, SkyTrie, Slot, TrieValue};

pub struct SkyTrieIterator {
    trie: SkyTrie,
    jobs: Vec<Job>,
}

impl SkyTrieIterator {
    fn from_trie(trie: SkyTrie) -> Self {
        let jobs = Job::start(&trie.get_root()).into_iter().collect();
        Self { trie, jobs }
    }
}

impl Iterator for SkyTrieIterator {
    type Item = (i32, TrieValue);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut job) = self.jobs.pop() {
            let base = self.trie.get_base(job.base, job.slot_count);
            match &base.as_ref()[job.slot_offset] {
                Slot::KeyValue(key_value) => {
                    let kv = key_value.to_trie_key_trie_value(&self.trie);
                    if job.next() {
                        self.jobs.push(job);
                    }
                    return Some(kv);
                }
                Slot::MapBase(lower_map_base) => {
                    let lower_job = Job::start(lower_map_base);
                    if job.next() {
                        self.jobs.push(job);
                    }
                    if let Some(lower_job) = lower_job {
                        self.jobs.push(lower_job);
                    }
                }
                Slot::ByteData(_) => unreachable!("base should contain no byte-data slots"),
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
    base: BufferIndex,
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
                base: map_base.base,
            })
        }
    }

    fn next(&mut self) -> bool {
        self.slot_offset += 1;
        self.slot_offset < self.slot_count
    }
}
