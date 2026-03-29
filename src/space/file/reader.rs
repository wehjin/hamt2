use crate::space::core::reader::SlotValue;
use crate::space::file::block::BlockTable;
use crate::space::file::details::Details;
use crate::space::TableAddr;
use crate::{space, ReadError};
use lru::LruCache;
use redb::{Database, ReadableDatabase};
use std::cell::RefCell;
use std::num::NonZeroUsize;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct DbReader {
    pub db: Rc<Database>,
    pub details: Details,
    pub lru: RefCell<LruCache<TableAddr, Vec<SlotValue>>>,
}

impl DbReader {
    pub(crate) fn new(db: Rc<Database>, details: Details) -> Self {
        Self {
            db,
            details,
            lru: RefCell::new(LruCache::new(NonZeroUsize::new(5).unwrap())),
        }
    }

    fn read_cache(&self, slot_addr: TableAddr) -> Option<SlotValue> {
        let lru = self.lru.borrow_mut();
        for (key, value) in lru.iter() {
            if &slot_addr >= key && slot_addr < (key + value.len()) {
                let value_offset = slot_addr - *key;
                return Some(value[value_offset]);
            }
        }
        None
    }

    fn fill_cache(&self, slot_addr: TableAddr) {
        let mut lru = self.lru.borrow_mut();
        let read = self.db.begin_read().expect("begin read");
        let entry = BlockTable::read_slots(&read, slot_addr);
        if let Some((block_start, block_slots)) = entry {
            let block_start = TableAddr(block_start);
            lru.put(block_start, block_slots);
        }
    }
}

impl space::Read for DbReader {
    fn read_slot(&self, addr: &TableAddr, offset: usize) -> Result<SlotValue, ReadError> {
        let slot_addr = addr + offset;
        if slot_addr >= self.details.max_addr() {
            return Err(ReadError::SlotAddressOutOfBounds(*addr, offset));
        }
        if let Some(slot) = self.read_cache(slot_addr) {
            return Ok(slot);
        }
        self.fill_cache(slot_addr);
        if let Some(slot) = self.read_cache(slot_addr) {
            Ok(slot)
        } else {
            Err(ReadError::NoSlotValueAtTableAddrOffset(slot_addr, offset))
        }
    }

    fn read_root(&self) -> Result<&Option<TableAddr>, ReadError> {
        Ok(&self.details.root)
    }
}
