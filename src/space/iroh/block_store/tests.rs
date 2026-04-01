use crate::space::block::store::{Block, BlockStore, Details};
use crate::space::core::reader::SlotValue;
use crate::space::iroh::block_store::IrohBlockStore;
use crate::space::iroh::client::IrohClient;
use crate::space::TableAddr;
use iroh::SecretKey;
use iroh_docs::NamespaceId;

#[tokio::test]
async fn read_and_writes_details() -> anyhow::Result<()> {
    let client = IrohClient::new_mem().await?;
    let doc = client.docs.create().await?;
    let store = IrohBlockStore::new(client, doc);
    let details = Details {
        slot_count: 30,
        root: Some(TableAddr::from(0x01020304u32)),
    };
    store.write_details(&details).await;
    assert_eq!(details, store.read_details().await);
    Ok(())
}

#[tokio::test]
async fn read_and_writes_blocks() -> anyhow::Result<()> {
    let client = IrohClient::new_mem().await?;
    let doc = client.docs.create().await?;
    let store = IrohBlockStore::new(client, doc);
    let block = Block {
        start_addr: TableAddr::from(0u32),
        slots: vec![SlotValue::from_u64(1)],
    };
    let details = Details {
        slot_count: block.slots.len(),
        root: None,
    };
    store.write_block_details(block.clone(), &details).await;
    let block2 = Block {
        start_addr: TableAddr::from(details.slot_count as u32),
        slots: vec![SlotValue::from_u64(2), SlotValue::from_u64(3)],
    };
    let details2 = Details {
        slot_count: details.slot_count + block2.slots.len(),
        root: None,
    };
    store.write_block_details(block2.clone(), &details2).await;
    assert_eq!(
        block,
        store
            .read_block(TableAddr::from(0u32))
            .await
            .expect("Block not found")
    );
    assert_eq!(
        block2,
        store
            .read_block(TableAddr::from(1u32))
            .await
            .expect("Block not found")
    );
    assert_eq!(
        block2,
        store
            .read_block(TableAddr::from(2u32))
            .await
            .expect("Block not found")
    );
    assert_eq!(None, store.read_block(TableAddr::from(3u32)).await);
    Ok(())
}

#[tokio::test]
async fn persistent_block_store_works() -> anyhow::Result<()> {
    let block = Block {
        start_addr: TableAddr::from(0u32),
        slots: vec![SlotValue::from_u64(1)],
    };
    let details = Details {
        slot_count: block.slots.len(),
        root: None,
    };

    let secret_key = SecretKey::from_bytes(&[0x01u8; 32]);
    let temp_dir = tempfile::tempdir()?;
    let doc_id: NamespaceId;
    {
        let client = IrohClient::connect(temp_dir.path(), secret_key.clone()).await?;
        let doc = client.docs.create().await?;
        doc_id = doc.id();
        let store = IrohBlockStore::new(client, doc);
        store.write_block_details(block.clone(), &details).await;
        assert_eq!(details, store.read_details().await);
        store.close().router.shutdown().await?;
    }
    {
        let client = IrohClient::connect(&temp_dir, secret_key.clone()).await?;
        let doc = client.docs.open(doc_id).await?.expect("doc should exist");
        let store = IrohBlockStore::new(client, doc);
        assert_eq!(details, store.read_details().await);
        assert_eq!(
            block,
            store
                .read_block(TableAddr::from(0u32))
                .await
                .expect("Block not found")
        );
        store.close().router.shutdown().await?;
    }
    Ok(())
}
