use crate::hamt::base::Change;
use crate::hamt::trie::mem::{MemMapBase, MemSlot, MemTrie};
use crate::hamt::trie::segment::{SegmentMapBase, SegmentTrie};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SegmentIndex(pub usize);

impl SegmentIndex {
    pub fn offset(&self, offset: usize) -> SegmentIndex {
        SegmentIndex(self.0 + offset)
    }
}

pub struct Segment {
    bytes: Vec<u8>,
}

impl Segment {
    pub fn new() -> Self {
        Self { bytes: vec![] }
    }

    pub fn write_trie(&mut self, trie: &MemTrie) -> usize {
        let map_base_start = self.write_map_base(&trie.map_base);
        map_base_start
    }

    pub fn read_trie(&self, map_base_start: SegmentIndex) -> SegmentTrie {
        let map_base = self.read_map_base(map_base_start);
        SegmentTrie { map_base }
    }

    pub fn write_map_base(&mut self, map_base: &MemMapBase) -> usize {
        let base_start = self.write_base(&map_base.base);
        let start = self.bytes.len();
        self.write_u32(map_base.map);
        self.write_u32(base_start as u32);
        start
    }

    pub fn read_map_base(&self, start: SegmentIndex) -> SegmentMapBase {
        let (map, base_part_start) = self.read_u32(start);
        let (base_start, _) = self.read_u32(base_part_start);
        SegmentMapBase {
            map,
            base_start: SegmentIndex(base_start as usize),
        }
    }

    pub fn write_base(&mut self, base: &[MemSlot]) -> usize {
        let start = self.bytes.len();
        for i in 0..base.len() {
            match &base[i] {
                MemSlot::Empty => panic!("Empty slot should never be written"),
                MemSlot::KeyValue(key, value) => {
                    self.write_u32(*key);
                    self.write_u32(*value);
                }
            }
        }
        start
    }

    pub fn read_slot(&self, base_index: usize, base_start: SegmentIndex) -> (u32, u32) {
        let slot_start = base_start.offset(base_index * 8);
        let (left, slot_part_start) = self.read_u32(slot_start);
        let (right, _) = self.read_u32(slot_part_start);
        (left, right)
    }

    pub fn write_changes(&mut self, changes: &[Change]) -> Vec<usize> {
        let count = changes.len() as u16;
        self.write_u16(count);
        let mut starts = vec![];
        for change in changes {
            starts.push(self.bytes.len());
            let bytes = change.to_be_bytes();
            self.bytes.extend_from_slice(&bytes);
        }
        starts
    }

    pub fn write_u32(&mut self, value: u32) {
        let bytes = value.to_be_bytes() as [u8; 4];
        self.bytes.extend_from_slice(&bytes);
    }

    pub fn read_u32(&self, start: SegmentIndex) -> (u32, SegmentIndex) {
        let end = start.0 + 4;
        let bytes = &self.bytes[start.0..end];
        let bytes = u32::from_be_bytes(bytes.try_into().expect("Invalid u32 bytes"));
        (bytes, SegmentIndex(end))
    }

    pub fn write_u16(&mut self, value: u16) {
        let bytes = value.to_be_bytes() as [u8; 2];
        self.bytes.extend_from_slice(&bytes);
    }

    pub fn read_u16(&self, start: SegmentIndex) -> (u16, SegmentIndex) {
        let end = start.0 + 2;
        let bytes = &self.bytes[start.0..end];
        let bytes = u16::from_be_bytes(bytes.try_into().expect("Invalid u16 bytes"));
        (bytes, SegmentIndex(end))
    }

    pub fn read_u8(&self, start: SegmentIndex) -> (u8, SegmentIndex) {
        let end = start.0 + 1;
        let bytes = &self.bytes[start.0..end];
        let bytes = bytes[0];
        (bytes, SegmentIndex(end))
    }
}