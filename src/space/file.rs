use crate::space::reader::SlotValue;
use crate::space::{Space, TableAddr};
use crate::{space, FileError, ReadError, TransactError};
use lru::LruCache;
use redb::{Database, ReadableDatabase, TableDefinition, WriteTransaction};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::num::NonZeroUsize;
use std::path::Path;
use std::rc::Rc;

const SLOTS_TABLE: TableDefinition<'static, u32, u64> = TableDefinition::new("slots");

#[derive(Debug, Clone)]
pub struct DbReader {
    db: Rc<Database>,
    details: Details,
    lru: RefCell<LruCache<TableAddr, SlotValue>>,
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub(crate) struct Details {
    slot_count: usize,
    root: Option<TableAddr>,
}

impl Details {
    pub fn max_addr(&self) -> TableAddr {
        TableAddr::from(self.slot_count)
    }

    pub fn with_update(&self, more_slots: usize, root: Option<TableAddr>) -> Self {
        Self {
            slot_count: self.slot_count + more_slots,
            root,
        }
    }

    const TABLE: TableDefinition<'static, &str, Vec<u8>> = TableDefinition::new("details");
    pub fn write(self: &Self, write: &WriteTransaction) -> Result<(), FileError> {
        let mut table = write.open_table(Self::TABLE)?;
        let bytes = postcard::to_allocvec(self)?;
        table.insert("main", bytes)?;
        Ok(())
    }
    pub fn read(db: &Database) -> Result<Self, FileError> {
        let read = db.begin_read()?;
        let table = read.open_table(Self::TABLE)?;
        let bytes = table.get("main")?.expect("get details").value();
        let details: Self = postcard::from_bytes(&bytes)?;
        Ok(details)
    }
}

#[derive(Debug)]
pub struct FileSpace {
    db: Rc<Database>,
    details: Details,
}

impl FileSpace {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, FileError> {
        let db = Database::create(path)?;
        let details = Details {
            slot_count: 0,
            root: None,
        };
        let write = db.begin_write()?;
        details.write(&write)?;
        write.commit()?;
        Ok(Self {
            db: Rc::new(db),
            details,
        })
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, FileError> {
        let db = Database::open(path)?;
        let details = Details::read(&db)?;
        Ok(Self {
            db: Rc::new(db),
            details,
        })
    }
    const SLOTS_TABLE: TableDefinition<'static, u32, u64> = TableDefinition::new("slots");
}
impl Space for FileSpace {
    type Reader = DbReader;

    fn add_segment(
        &mut self,
        start_addr: TableAddr,
        slots: Vec<SlotValue>,
        root: Option<TableAddr>,
    ) -> Result<(), TransactError> {
        if start_addr != self.max_addr() {
            return Err(TransactError::InvalidStartAddr(start_addr));
        }

        let new_details: Details;
        {
            let write = self.db.begin_write().expect("begin write");
            {
                let mut table = write
                    .open_table(FileSpace::SLOTS_TABLE)
                    .expect("open slots table");
                let mut key = start_addr.0;
                for slot in &slots {
                    let value = slot.to_u64();
                    table.insert(key, value).expect("insert slot");
                    key += 1;
                }
            }
            new_details = self.details.with_update(slots.len(), root);
            new_details.write(&write).expect("write details");
            write.commit().expect("commit");
        }
        self.details = new_details;
        Ok(())
    }
    fn read(&self) -> Result<Self::Reader, ReadError> {
        let reader = DbReader::new(self.db.clone(), self.details.clone());
        Ok(reader)
    }
    fn max_addr(&self) -> TableAddr {
        self.details.max_addr()
    }
}
