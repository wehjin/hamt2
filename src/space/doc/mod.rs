use crate::space::core::block_space::reader::BlockReader;
use crate::space::core::block_space::BlockSpace;
use crate::space::core::reader::SlotValue;
use crate::space::doc::block_store::DocBlockStore;
use crate::space::doc::client::DocsClient;
use crate::space::{Space, TableAddr};
use crate::{FileError, ReadError, TransactError};
use iroh_docs::NamespaceId;

pub mod block_store;
pub mod client;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct DocSpace {
    block_space: BlockSpace<DocBlockStore>,
    doc_id: NamespaceId,
}

impl DocSpace {
    pub async fn new(client: DocsClient) -> Result<Self, FileError> {
        let doc = client.docs.create().await?;
        let doc_id = doc.id();
        let block_store = DocBlockStore::new(client, doc);
        let block_space = BlockSpace::new(block_store).await?;
        let space = Self {
            block_space,
            doc_id,
        };
        Ok(space)
    }

    pub async fn load(client: DocsClient, doc_id: NamespaceId) -> Result<Self, FileError> {
        let doc = client.docs.open(doc_id).await?.expect("doc not found");
        let block_store = DocBlockStore::new(client, doc);
        let block_space = BlockSpace::load(block_store).await?;
        let space = Self {
            block_space,
            doc_id,
        };
        Ok(space)
    }

    pub fn doc_id(&self) -> NamespaceId {
        self.doc_id
    }

    pub async fn close(self) -> anyhow::Result<()> {
        let client = self.block_space.close().close();
        client.router.shutdown().await?;
        Ok(())
    }
}

impl Space for DocSpace {
    type Reader = BlockReader<DocBlockStore>;

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
