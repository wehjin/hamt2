use crate::space::core::reader::SlotValue;
use crate::space::{Space, TableAddr};
use crate::{FileError, ReadError, TransactError};
use details::Details;
use reader::DbReader;
use redb::{Database, TableDefinition};
use std::path::Path;
use std::rc::Rc;

pub mod details;
pub mod reader;

const SLOTS_TABLE: TableDefinition<'static, u32, u64> = TableDefinition::new("slots");

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
