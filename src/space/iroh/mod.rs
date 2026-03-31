use crate::space::block::store::{Block, BlockStore, Details};
use crate::space::core::reader::SlotValue;
use crate::space::iroh::client::IrohClient;
use crate::space::TableAddr;
use iroh_docs::store::Query;

pub mod client;
#[cfg(test)]
mod tests;

pub struct IrohBlockStore {
    client: IrohClient,
}

impl IrohBlockStore {
    pub fn new(client: IrohClient) -> Self {
        Self { client }
    }

    const DETAILS_KEY: &'static str = "DETAILS";
}

impl BlockStore for IrohBlockStore {
    async fn write_details(&self, details: &Details) {
        let card = postcard::to_allocvec(details).expect("Failed to serialize details");
        let card_tag = self
            .client
            .store
            .add_slice(card.as_slice())
            .await
            .expect("Failed to add slice");
        self.client
            .doc
            .set_hash(
                self.client.author,
                Self::DETAILS_KEY,
                card_tag.hash,
                card.len() as u64,
            )
            .await
            .expect("Failed to set hash");
    }

    async fn write_block_details(&self, block: Block, details: &Details) {
        let Block { start_addr, slots } = block;
        let slots_bytes = slots_into_bytes(slots);
        let slots_tag = self
            .client
            .store
            .add_slice(slots_bytes.as_slice())
            .await
            .expect("Failed to add slice");
        let key = table_addr_into_bytes(start_addr);
        self.client
            .doc
            .set_hash(
                self.client.author,
                key,
                slots_tag.hash,
                slots_bytes.len() as u64,
            )
            .await
            .expect("Failed to set hash");
        self.write_details(details).await;
    }

    async fn read_block(&self, addr: TableAddr) -> Option<Block> {
        let addr_u32 = addr.u32();
        let key = table_addr_into_bytes(addr);
        todo!()
    }

    async fn read_details(&self) -> Details {
        let query = Query::single_latest_per_key().key_exact(Self::DETAILS_KEY);
        let result = self
            .client
            .doc
            .get_one(query)
            .await
            .expect("Failed to get details")
            .expect("Details not found");
        let hash = result.content_hash();
        let bytes = self
            .client
            .store
            .get_bytes(hash)
            .await
            .expect("Failed to get details");
        let details =
            postcard::from_bytes::<Details>(bytes.as_ref()).expect("Failed to deserialize details");
        details
    }
}

fn table_addr_into_bytes(addr: TableAddr) -> Vec<u8> {
    addr.u32().to_be_bytes().as_slice().to_vec()
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
