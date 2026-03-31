use crate::space::block::reader::BlockReader;
use crate::space::block::BlockSpace;
use crate::space::core::reader::SlotValue;
use crate::space::file::block_store::RedBlockStore;
use crate::space::{Space, TableAddr};
use crate::{FileError, ReadError, TransactError};
use redb::Database;
use std::fmt::Debug;
use std::path::Path;

pub mod block_store;
pub mod block_table;
pub mod details_table;
#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct FileSpace {
    block_space: BlockSpace<RedBlockStore>,
}

impl FileSpace {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self, FileError> {
        let db = Database::create(path)?;
        let block_store = RedBlockStore::new(db);
        let block_space = BlockSpace::new(block_store).await?;
        let red_space = Self { block_space };
        Ok(red_space)
    }

    pub async fn load(path: impl AsRef<Path>) -> Result<Self, FileError> {
        let db = Database::open(path)?;
        let block_store = RedBlockStore::new(db);
        let block_space = BlockSpace::load(block_store).await?;
        let red_space = Self { block_space };
        Ok(red_space)
    }
}
impl Space for FileSpace {
    type Reader = BlockReader<RedBlockStore>;

    async fn add_segment(
        &mut self,
        start_addr: TableAddr,
        slots: Vec<SlotValue>,
        root: Option<TableAddr>,
    ) -> Result<(), TransactError> {
        self.block_space.add_segment(start_addr, slots, root).await
    }
    async fn read(&self) -> Result<Self::Reader, ReadError> {
        self.block_space.read().await
    }
    fn max_addr(&self) -> TableAddr {
        self.block_space.max_addr()
    }
}
