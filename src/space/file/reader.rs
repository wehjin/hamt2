use crate::space::core::reader::SlotValue;
use crate::space::file::block_store::RedBlockStore;
use crate::space::core::block_store::Details;
use crate::space::TableAddr;
use crate::{space, ReadError};
use lru::LruCache;
use std::cell::RefCell;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use crate::space::core::block_store::{Block, BlockStore};

#[derive(Debug, Clone)]
pub struct DbReader {
    block_store: RedBlockStore,
    details: Details,
    lru: RefCell<LruCache<TableAddr, Vec<SlotValue>>>,
}

impl DbReader {
    pub(crate) fn new(block_store: RedBlockStore, details: Details) -> Self {
        Self {
            block_store,
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
        let block = self.block_store.read_block(slot_addr);
        if let Some(Block { start_addr, slots }) = block {
            lru.put(start_addr, slots);
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
