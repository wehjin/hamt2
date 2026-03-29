use std::cell::RefCell;
use std::num::NonZeroUsize;
use std::rc::Rc;
use redb::{Database, ReadableDatabase};
use lru::LruCache;
use crate::{space, ReadError};
use crate::space::core::reader::SlotValue;
use crate::space::file::details::Details;
use crate::space::file::SLOTS_TABLE;
use crate::space::TableAddr;

#[derive(Debug, Clone)]
pub struct DbReader {
    pub db: Rc<Database>,
    pub details: Details,
    pub lru: RefCell<LruCache<TableAddr, SlotValue>>,
}

impl DbReader {
    pub(crate) fn new(db: Rc<Database>, details: Details) -> Self {
        Self {
            db,
            details,
            lru: RefCell::new(LruCache::new(NonZeroUsize::new(1000).unwrap())),
        }
    }
}

impl space::Read for DbReader {
    fn read_slot(&self, addr: &TableAddr, offset: usize) -> Result<SlotValue, ReadError> {
        let slot_addr = addr + offset;
        if slot_addr > self.details.max_addr() {
            return Err(ReadError::InvalidTableAddr(*addr));
        }
        if let Some(slot) = self.lru.borrow_mut().get(&slot_addr) {
            Ok(slot.clone())
        } else {
            let read = self.db.begin_read().expect("begin read");
            let table = read.open_table(SLOTS_TABLE).expect("open slots table");
            let slot_u64 = table
                .get(slot_addr.u32())
                .expect("get slot")
                .expect("get slot value")
                .value();
            let slot = SlotValue::from(slot_u64);
            self.lru.borrow_mut().put(slot_addr, slot);
            Ok(slot)
        }
    }

    fn read_root(&self) -> Result<&Option<TableAddr>, ReadError> {
        Ok(&self.details.root)
    }
}