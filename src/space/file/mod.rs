use crate::space::core::block_store::Details;
use crate::space::core::block_store::{Block, BlockStore};
use crate::space::core::reader::SlotValue;
use crate::space::file::block_store::RedBlockStore;
use crate::space::{Space, TableAddr};
use crate::{FileError, ReadError, TransactError};
use reader::DbReader;
use redb::Database;
use std::path::Path;

pub mod block_store;
pub mod block_table;
pub mod details_table;
pub mod reader;
#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct FileSpace {
    block_store: RedBlockStore,
    details: Details,
}

impl FileSpace {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, FileError> {
        let db = Database::create(path)?;
        let block_store = RedBlockStore::new(db);
        let details = Details {
            slot_count: 0,
            root: None,
        };
        block_store.write_details(&details);
        let space = Self {
            block_store,
            details,
        };
        Ok(space)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, FileError> {
        let db = Database::open(path)?;
        let block_store = RedBlockStore::new(db);
        let details = block_store.read_details();
        let space = Self {
            block_store,
            details,
        };
        Ok(space)
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
        let block = Block { start_addr, slots };
        self.block_store.write_block_details(block, &new_details);
        self.details = new_details;
        Ok(())
    }
    fn read(&self) -> Result<Self::Reader, ReadError> {
        let reader = DbReader::new(self.block_store.clone(), self.details.clone());
        Ok(reader)
    }
    fn max_addr(&self) -> TableAddr {
        self.details.max_addr()
    }
}
