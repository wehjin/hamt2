use crate::space::block::store::Details;
use crate::space::block::store::{Block, BlockStore};
use crate::space::core::reader::SlotValue;
use crate::space::TableAddr;
use crate::{space, ReadError};
use lru::LruCache;
use std::cell::RefCell;
use std::fmt::Debug;
use std::num::NonZeroUsize;

#[derive(Debug, Clone)]
pub struct BlockReader<T: BlockStore + Debug + Clone> {
    block_store: T,
    details: Details,
    lru: RefCell<LruCache<TableAddr, Vec<SlotValue>>>,
}

impl<T: BlockStore + Debug + Clone> BlockReader<T> {
    pub(crate) fn new(block_store: T, details: Details) -> Self {
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

impl<T: BlockStore + Debug + Clone> space::Read for BlockReader<T> {
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
