use crate::error::ReadError;
use crate::space::{Read, TableAddr};
use serde::{Deserialize, Serialize};
use std::ops::Index;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlotValue(pub u32, pub u32);

impl SlotValue {
    pub fn left(&self) -> u32 {
        self.0
    }
    pub fn right(&self) -> u32 {
        self.1
    }
}

#[derive(Debug, Clone)]
pub struct SlotTable {
    slots: Vec<SlotValue>,
}

impl SlotTable {
    pub fn new() -> Self {
        Self { slots: Vec::new() }
    }

    pub fn max_index(&self) -> TableAddr {
        TableAddr(self.slots.len() as u32)
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn extend<T: IntoIterator<Item = SlotValue>>(&mut self, iter: T) {
        self.slots.extend(iter);
    }

    pub fn into_slots(self) -> Vec<SlotValue> {
        self.slots
    }
}

impl From<u64> for SlotValue {
    fn from(value: u64) -> Self {
        let left = (value >> 32) as u32;
        let right = (value & 0xffff_ffff) as u32;
        Self(left, right)
    }
}

impl Index<TableAddr> for SlotTable {
    type Output = SlotValue;

    fn index(&self, index: TableAddr) -> &Self::Output {
        &self.slots[index.0 as usize]
    }
}

#[derive(Debug, Clone)]
pub struct MemReader {
    slots: Rc<SlotTable>,
    root: Option<TableAddr>,
}

impl MemReader {
    pub fn new(slots: SlotTable, root: Option<TableAddr>) -> Self {
        Self {
            slots: Rc::new(slots),
            root,
        }
    }
}

impl Read for MemReader {
    fn read_slot(&self, addr: &TableAddr, offset: usize) -> Result<SlotValue, ReadError> {
        let offset_addr = addr + offset;
        if offset_addr >= self.slots.max_index() {
            Err(ReadError::InvalidTableAddr(*addr))
        } else {
            Ok(self.slots[offset_addr])
        }
    }

    fn read_root(&self) -> Result<&Option<TableAddr>, ReadError> {
        Ok(&self.root)
    }
}
