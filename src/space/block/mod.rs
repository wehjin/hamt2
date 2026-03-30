use crate::space::block::store::{Block, BlockStore, Details};
use crate::space::core::reader::SlotValue;
use reader::BlockReader;
use crate::space::{Space, TableAddr};
use crate::{FileError, ReadError, TransactError};
use std::fmt::Debug;

pub mod store;
pub mod reader;

#[derive(Debug)]
pub struct BlockSpace<T: BlockStore + Debug> {
    block_store: T,
    details: Details,
}

impl<T: BlockStore + Debug> BlockSpace<T> {
    pub fn new(block_store: T) -> Result<Self, FileError> {
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

    pub fn load(block_store: T) -> Result<Self, FileError> {
        let details = block_store.read_details();
        let space = Self {
            block_store,
            details,
        };
        Ok(space)
    }
}

impl<T: BlockStore + Debug + Clone> Space for BlockSpace<T> {
    type Reader = BlockReader<T>;

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
        let reader = BlockReader::new(self.block_store.clone(), self.details.clone());
        Ok(reader)
    }
    fn max_addr(&self) -> TableAddr {
        self.details.max_addr()
    }
}
