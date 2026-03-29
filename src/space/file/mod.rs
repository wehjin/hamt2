use crate::space::core::reader::SlotValue;
use crate::space::file::block::BlockTable;
use crate::space::{Space, TableAddr};
use crate::{FileError, ReadError, TransactError};
use details::Details;
use reader::DbReader;
use redb::Database;
use std::path::Path;
use std::rc::Rc;

pub mod block;
pub mod details;
pub mod reader;
#[cfg(test)]
mod tests;

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
        let new_details = self.details.with_update(slots.len(), root);
        let write = self.db.begin_write().expect("begin write");
        {
            BlockTable::write_slots(&write, start_addr, slots);
            new_details.write(&write).expect("write details");
        }
        write.commit().expect("commit");
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
