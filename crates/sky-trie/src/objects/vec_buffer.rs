use crate::{Base, Buffer, BufferIndex, BufferMut, MapBase, Slot};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct VecBuffer {
    past: Option<Arc<VecBuffer>>,
    slots: Vec<Slot>,
}

impl VecBuffer {
    pub fn new(seed: Slot) -> Self {
        Self {
            past: None,
            slots: vec![seed],
        }
    }
    pub fn extend(past: Arc<Self>) -> Self {
        Self {
            past: Some(past),
            slots: vec![],
        }
    }
    pub fn retract(self) -> Self {
        if let Some(past) = self.past {
            let mut slots = past.deref().slots.clone();
            slots.extend(self.slots);
            debug_assert!(matches!(dbg!(slots.last()), Some(Slot::MapBase(_))));
            Self { past: None, slots }
        } else {
            self
        }
    }
    pub fn len(&self) -> usize {
        self.slots.len() + self.past_len()
    }
    fn past_len(&self) -> usize {
        if let Some(past) = &self.past {
            past.slots.len()
        } else {
            0usize
        }
    }
}

impl Buffer for VecBuffer {
    fn max_index(&self) -> BufferIndex {
        let len = self.len();
        BufferIndex(len as i32 - 1)
    }

    fn get_base(&self, id: BufferIndex, size: usize) -> Base {
        if id.0 < 0 {
            return Base::empty();
        }
        let mut start = id.0 as usize;
        if let Some(past) = &self.past {
            let past_len = past.slots.len();
            if start < past_len {
                return past.get_base(id, size);
            }
            start -= past_len;
        }
        if start >= self.slots.len() {
            return Base::empty();
        }
        let end = start + size;
        debug_assert!(end <= self.slots.len(), "too few slots in the buffer");
        let slots = &self.slots[start..end];
        Base::new(size, slots)
    }
}

impl BufferMut for VecBuffer {
    async fn push_root(&mut self, root: MapBase) {
        let slot = Slot::MapBase(root);
        self.slots.push(slot);
    }

    async fn push_base(&mut self, base: Base) -> BufferIndex {
        let index = self.next_index();
        self.slots.extend(base.slots);
        index
    }
}
