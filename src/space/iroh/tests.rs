use crate::space::core::reader::SlotValue;
use crate::space::iroh::client::IrohClient;
use crate::space::iroh::IrohSpace;
use crate::space::{Read, Space, TableAddr};
use iroh::SecretKey;

#[tokio::test]
async fn persistent_space_works() -> anyhow::Result<()> {
    let secret_key = SecretKey::from_bytes(&[0x01u8; 32]);
    let temp_dir = tempfile::tempdir()?;
    let mut doc_id = None;
    {
        let client = IrohClient::connect(temp_dir.path(), doc_id, secret_key.clone()).await?;
        doc_id = Some(client.doc.id());
        let mut space = IrohSpace::new(client).await?;
        {
            let mut extend = space.extend().await?;
            extend.add_slots(vec![SlotValue::from_u64(8)]);
            extend.commit(&mut space).await?;
        }
        space.close().router.shutdown().await?;
    }
    {
        let client = IrohClient::connect(&temp_dir, doc_id, secret_key.clone()).await?;
        let space = IrohSpace::load(client).await?;
        let reader = space.read().await?;
        assert_eq!(
            SlotValue::from_u64(8),
            reader.read_slot(&TableAddr::from(0u32), 0).await?
        );
        space.close().router.shutdown().await?;
    }
    Ok(())
}

#[tokio::test]
async fn iroh_space_works() -> anyhow::Result<()> {
    let client = IrohClient::new_mem().await?;
    {
        let mut space = IrohSpace::new(client.clone()).await?;
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
        let space = IrohSpace::load(client).await.expect("load red space");
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
