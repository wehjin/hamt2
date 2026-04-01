use crate::space::core::reader::SlotValue;
use crate::space::doc::client::DocsClient;
use crate::space::doc::DocSpace;
use crate::space::{Read, Space, TableAddr};
use iroh::SecretKey;
use iroh_docs::NamespaceId;

#[tokio::test]
async fn memory_doc_space_works() -> anyhow::Result<()> {
    let client = DocsClient::new_mem().await?;
    let doc_id: NamespaceId;
    {
        let mut space = DocSpace::new(client.clone()).await?;
        doc_id = space.doc_id;
        assert_eq!(TableAddr::ZERO, space.max_addr());
        {
            let mut extend = space.extend().await?;
            extend.add_slots(vec![SlotValue::from_u64(1)]);
            extend.commit(&mut space).await?;
        }
        {
            let mut extend = space.extend().await?;
            extend.add_slots(vec![
                SlotValue::from_u64(5),
                SlotValue::from_u64(6),
                SlotValue::from_u64(7),
            ]);
            extend.commit(&mut space).await?;
        }
        assert_eq!(TableAddr::from(4usize), space.max_addr());
    }
    {
        let space = DocSpace::load(client, doc_id)
            .await
            .expect("load red space");
        let reader = space.read().await?;
        assert_eq!(
            SlotValue::from_u64(1),
            reader.read_slot(&TableAddr::from(0u32), 0).await?
        );
        assert_eq!(
            SlotValue::from_u64(5),
            reader.read_slot(&TableAddr::from(0u32), 1).await?
        );
        assert_eq!(
            SlotValue::from_u64(5),
            reader.read_slot(&TableAddr::from(1u32), 0).await?
        );
        assert_eq!(
            SlotValue::from_u64(6),
            reader.read_slot(&TableAddr::from(1u32), 1).await?
        );
        assert_eq!(
            SlotValue::from_u64(7),
            reader.read_slot(&TableAddr::from(1u32), 2).await?
        );
    }

    Ok(())
}

#[tokio::test]
async fn persistent_doc_space_works() -> anyhow::Result<()> {
    let secret_key = SecretKey::from_bytes(&[0x01u8; 32]);
    let temp_dir = tempfile::tempdir()?;
    let doc_id: NamespaceId;
    {
        let client = DocsClient::connect(temp_dir.path(), secret_key.clone()).await?;
        let mut space = DocSpace::new(client).await?;
        doc_id = space.doc_id();
        {
            let mut extend = space.extend().await?;
            extend.add_slots(vec![SlotValue::from_u64(8)]);
            extend.commit(&mut space).await?;
        }
        space.close().await?
    }
    {
        let client = DocsClient::connect(&temp_dir, secret_key.clone()).await?;
        let space = DocSpace::load(client, doc_id).await?;
        let reader = space.read().await?;
        assert_eq!(
            SlotValue::from_u64(8),
            reader.read_slot(&TableAddr::from(0u32), 0).await?
        );
        space.close().await?
    }
    Ok(())
}
