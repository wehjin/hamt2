use crate::space::block::store::{Block, BlockStore, Details};
use crate::space::core::reader::SlotValue;
use crate::space::iroh::block_store::block_key::BlockKey;
use crate::space::iroh::block_store::iroh_key::IrohKey;
use crate::space::iroh::block_store::search_key::SearchKey;
use crate::space::iroh::client::IrohClient;
use crate::space::TableAddr;
use bytes::Bytes;
use iroh_docs::api::Doc;
use iroh_docs::store::{Query, SortDirection};
use tokio_stream::StreamExt;

mod block_key;
mod iroh_key;
mod search_key;

#[derive(Debug, Clone)]
pub struct IrohBlockStore {
    client: IrohClient,
    doc: Doc,
}

impl IrohBlockStore {
    pub fn new(client: IrohClient, doc: Doc) -> Self {
        Self { client, doc }
    }
    pub fn close(self) -> IrohClient {
        self.client
    }
}

impl BlockStore for IrohBlockStore {
    async fn write_details(&self, details: &Details) {
        let card = postcard::to_allocvec(details).expect("Failed to serialize details");
        self.doc
            .set_bytes(self.client.author, IrohKey::Details, card)
            .await
            .expect("Failed to set details");
    }

    async fn read_details(&self) -> Details {
        let key_bytes: Bytes = IrohKey::Details.into();
        let query = Query::single_latest_per_key().key_exact(key_bytes.as_ref());
        let details_entry = self
            .doc
            .get_one(query)
            .await
            .expect("Failed to get details")
            .expect("Details not found");
        let details_hash = details_entry.content_hash();
        let details_bytes = self
            .client
            .store
            .get_bytes(details_hash)
            .await
            .expect("Failed to get details bytes");
        let details = postcard::from_bytes::<Details>(details_bytes.as_ref())
            .expect("Failed to deserialize details");
        details
    }

    async fn write_block_details(&self, block: Block, details: &Details) {
        let Block { start_addr, slots } = block;
        let key = IrohKey::Block(BlockKey::new(start_addr, slots.len() as u32));
        let key_bytes: Bytes = key.into();
        let slot_bytes = slots_into_bytes(slots);
        self.doc
            .set_bytes(self.client.author, key_bytes, slot_bytes)
            .await
            .expect("Failed to set block");
        self.write_details(details).await;
    }

    async fn read_block(&self, addr: TableAddr) -> Option<Block> {
        let mut search_queue = Some(IrohKey::Search(SearchKey::from_addr(&addr)));
        while let Some(iroh_search) = search_queue.take() {
            let prefix_bytes: Bytes = iroh_search.into();
            let query = Query::single_latest_per_key()
                .key_prefix(prefix_bytes.as_ref())
                .sort_direction(SortDirection::Desc);
            let results = self.doc.get_many(query).await.expect("Failed to get block");
            tokio::pin!(results);
            while let Some(entry) = results.next().await {
                let entry = entry.expect("Failed to get block entry");
                let block_key = IrohKey::from(entry.key().as_ref()).into_block_key();
                if block_key.handles_addr(addr) {
                    let start_addr = block_key.to_addr();
                    let slots_hash = entry.content_hash();
                    let slot_bytes = self.client.store.get_bytes(slots_hash).await.ok()?;
                    let slots = slots_from_bytes(slot_bytes);
                    let block = Block { start_addr, slots };
                    return Some(block);
                }
            }
            search_queue = iroh_search.into_search_key().next().map(IrohKey::Search);
        }
        None
    }
}
fn slots_into_bytes(slots: Vec<SlotValue>) -> Vec<u8> {
    let slots_count = slots.len();
    let empty = Vec::with_capacity(slots_count * size_of::<u64>());
    slots.into_iter().fold(empty, |mut acc, slot| {
        let slot_u64 = slot.to_u64();
        let slot_bytes = slot_u64.to_be_bytes();
        acc.extend_from_slice(&slot_bytes);
        acc
    })
}

fn slots_from_bytes(bytes: Bytes) -> Vec<SlotValue> {
    let slots_count = bytes.len() / size_of::<u64>();
    let mut slots = Vec::with_capacity(slots_count);
    for i in 0..slots_count {
        let slot_bytes = bytes.get(i * size_of::<u64>()..(i + 1) * size_of::<u64>());
        let slot_u64 = u64::from_be_bytes(slot_bytes.unwrap().try_into().unwrap());
        slots.push(SlotValue::from_u64(slot_u64));
    }
    slots
}

#[cfg(test)]
mod tests;
