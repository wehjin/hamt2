use crate::space::block::reader::BlockReader;
use crate::space::block::BlockSpace;
use crate::space::core::reader::SlotValue;
use crate::space::iroh::block_store::IrohBlockStore;
use crate::space::iroh::client::IrohClient;
use crate::space::{Space, TableAddr};
use crate::{FileError, ReadError, TransactError};

pub mod block_store;
pub mod client;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct IrohSpace {
    block_space: BlockSpace<IrohBlockStore>,
}

impl IrohSpace {
    pub async fn new(client: IrohClient) -> Result<Self, FileError> {
        let block_store = IrohBlockStore::new(client);
        let block_space = BlockSpace::new(block_store).await?;
        let space = Self { block_space };
        Ok(space)
    }

    pub async fn load(client: IrohClient) -> Result<Self, FileError> {
        let block_store = IrohBlockStore::new(client);
        let block_space = BlockSpace::load(block_store).await?;
        let space = Self { block_space };
        Ok(space)
    }

    pub fn close(self) -> IrohClient {
        self.block_space.close().close()
    }
}

impl Space for IrohSpace {
    type Reader = BlockReader<IrohBlockStore>;

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
