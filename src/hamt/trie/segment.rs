use std::rc::Rc;
use crate::hamt::trie::mem::MemSlot;
use crate::hamt::segment::{Segment, SegmentIndex};
use crate::hamt::trie::key::TrieKey;

pub struct SegmentTrie {
    pub map_base: SegmentMapBase,
}

impl SegmentTrie {
    pub fn query_value(&self, key: TrieKey, segment: Rc<Segment>) -> Option<u32> {
        match self.map_base.load_slot(key, segment) {
            MemSlot::Empty => None,
            MemSlot::KeyValue(_key, value) => Some(value),
        }
    }
}

pub struct SegmentMapBase {
    pub map: u32,
    pub base_start: SegmentIndex,
}

impl SegmentMapBase {
    pub fn load_slot(&self, key: TrieKey, segment: Rc<Segment>) -> MemSlot {
        let base_index = key.to_base_index(self.map);
        let (left, right) = segment.read_slot(base_index, self.base_start);
        match left == key.u32() {
            true => MemSlot::KeyValue(left, right),
            false => MemSlot::Empty,
        }
    }
}