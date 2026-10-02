use crate::{Base, Buffer, BufferIndex, BufferMut, MapBase, Slot};

mod edit;
mod view;

pub use edit::*;
pub use view::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SlotBuffer {
    slots: Vec<Slot>,
    root_index: Option<usize>,
}

impl SlotBuffer {
    pub fn new() -> Self {
        Self {
            slots: vec![],
            root_index: None,
        }
    }
    pub fn append(&mut self, novel_slots: SlotBuffer) {
        let legacy_slots_count = self.slots.len();
        if let Some(novel_root) = novel_slots.root_index {
            self.root_index = Some(novel_root + legacy_slots_count);
        }
        self.slots.extend(novel_slots.slots);
    }
    pub fn len(&self) -> usize {
        self.slots.len()
    }
}

impl Buffer for SlotBuffer {
    fn max_index(&self) -> BufferIndex {
        BufferIndex(self.slots.len() as i32 - 1)
    }

    fn get_root(&self) -> MapBase {
        if let Some(root_index) = self.root_index {
            let Slot::MapBase(map_base) = self.slots[root_index] else {
                panic!("expected a subtrie in the last slot")
            };
            map_base
        } else {
            MapBase::empty()
        }
    }

    async fn get_base(&self, id: BufferIndex, size: usize) -> Base {
        if id.0 < 0 || id.0 > self.slots.len() as i32 {
            return Base::empty();
        }
        let start = id.0 as usize;
        let end = start + size;
        assert!(end <= self.slots.len(), "too few slots in the buffer");
        let slots = &self.slots[start..end];
        Base::new(size, slots)
    }
}

impl BufferMut for SlotBuffer {
    async fn push_root(&mut self, root: MapBase) {
        self.slots.push(Slot::MapBase(root));
        self.root_index = Some(self.slots.len() - 1);
    }

    async fn push_base(&mut self, base: Base) -> BufferIndex {
        let index = self.next_index();
        self.slots.extend(base.slots);
        index
    }
}
