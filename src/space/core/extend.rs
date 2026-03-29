use crate::space::core::reader::{SlotTable, SlotValue};
use crate::space::TableAddr;
use crate::space::{Read, Space};
use crate::{ReadError, TransactError};

pub struct Extend<T: Space> {
    reader: T::Reader,
    start_addr: TableAddr,
    slots: SlotTable,
    root: Option<TableAddr>,
}

impl<T: Space> Extend<T> {
    pub fn new(start_addr: TableAddr, reader: T::Reader) -> Self {
        Self {
            reader,
            start_addr,
            slots: SlotTable::new(),
            root: None,
        }
    }

    fn max_addr(&self) -> TableAddr {
        self.start_addr + self.slots.len()
    }

    pub fn add_slots(&mut self, items: Vec<SlotValue>) -> TableAddr {
        let pos = self.max_addr();
        self.slots.extend(items);
        pos
    }

    pub fn set_root(&mut self, root: TableAddr) {
        self.root = Some(root);
    }

    pub fn commit(self, space: &mut T) -> Result<(), TransactError> {
        space.add_segment(self.start_addr, self.slots.into_slots(), self.root)?;
        Ok(())
    }
}

impl<T: Space> Read for Extend<T> {
    fn read_slot(&self, addr: &TableAddr, offset: usize) -> Result<SlotValue, ReadError> {
        let offset_addr = addr + offset;
        if offset_addr >= self.max_addr() {
            let index = offset_addr - self.start_addr;
            if index >= self.slots.len() {
                return Err(ReadError::InvalidTableAddr(offset_addr));
            }
            let slot = self.slots[index];
            Ok(slot)
        } else {
            self.reader.read_slot(addr, offset)
        }
    }

    fn read_root(&self) -> Result<&Option<TableAddr>, ReadError> {
        if self.root.is_some() {
            Ok(&self.root)
        } else {
            self.reader.read_root()
        }
    }
}
