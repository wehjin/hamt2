use crate::space::core::block_space::store::{Block, BlockStore, Details};
use crate::space::core::reader::SlotValue;
use crate::space::{Space, TableAddr};
use crate::{FileError, ReadError, TransactError};
use reader::BlockReader;
use std::fmt::Debug;

pub mod reader;
pub mod store;

#[derive(Debug)]
pub struct BlockSpace<T: BlockStore + Debug> {
    block_store: T,
    details: Details,
}

impl<T: BlockStore + Debug> BlockSpace<T> {
    pub async fn new(block_store: T) -> Result<Self, FileError> {
        let details = Details {
            slot_count: 0,
            root: None,
        };
        block_store.write_details(&details).await;
        let space = Self {
            block_store,
            details,
        };
        Ok(space)
    }

    pub async fn load(block_store: T) -> Result<Self, FileError> {
        let details = block_store.read_details().await;
        let space = Self {
            block_store,
            details,
        };
        Ok(space)
    }
    pub fn close(self) -> T {
        self.block_store
    }
}

impl<T: BlockStore + Debug + Clone> Space for BlockSpace<T> {
    type Reader = BlockReader<T>;

    async fn add_segment(
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
        self.block_store
            .write_block_details(block, &new_details)
            .await;
        self.details = new_details;
        Ok(())
    }
    async fn read(&self) -> Result<Self::Reader, ReadError> {
        let reader = BlockReader::new(self.block_store.clone(), self.details.clone());
        Ok(reader)
    }
    fn max_addr(&self) -> TableAddr {
        self.details.max_addr()
    }
}
