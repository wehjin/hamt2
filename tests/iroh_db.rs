use hamt2::db::{Attr, Datom, Db, Ent, Val};
use hamt2::space::iroh::client::IrohClient;
use hamt2::space::iroh::IrohSpace;
use iroh::SecretKey;
use iroh_docs::NamespaceId;

const ATTR_COUNT: Attr = Attr("counter", "count");

#[tokio::test]
async fn iroh_mem_db_works() -> anyhow::Result<()> {
    let client = IrohClient::new_mem().await?;
    let space = IrohSpace::new(client).await?;
    let mut db = Db::new(space, vec![ATTR_COUNT]).await?;
    db = db
        .transact(vec![Datom::Add(Ent::from(1), ATTR_COUNT, Val::U32(1))])
        .await?;
    let val = db.find_val(Ent::from(1), ATTR_COUNT).await?;
    assert_eq!(Some(Val::U32(1)), val);
    Ok(())
}

#[tokio::test]
async fn iroh_path_db_works() -> anyhow::Result<()> {
    let secret_key = SecretKey::from_bytes(&[0x01u8; 32]);
    let temp_dir = dbg!(tempfile::tempdir()?.keep());
    let doc_id: NamespaceId;
    {
        let client = IrohClient::connect(&temp_dir, secret_key.clone()).await?;
        let space = IrohSpace::new(client).await?;
        doc_id = space.doc_id;
        let db = Db::new(space, vec![ATTR_COUNT])
            .await?
            .transact(vec![Datom::Add(Ent::from(1), ATTR_COUNT, Val::U32(1))])
            .await?;
        let space = db.close();
        let client = space.close();
        client.router.shutdown().await?;
    }
    {
        let client = IrohClient::connect(&temp_dir, secret_key.clone()).await?;
        let space = IrohSpace::load(client, doc_id).await?;
        let db = Db::load(space, vec![ATTR_COUNT]).await?;
        assert_eq!(
            Some(Val::U32(1)),
            db.find_val(Ent::from(1), ATTR_COUNT).await?
        );
        let space = db.close();
        let client = space.close();
        client.router.shutdown().await?;
    }
    Ok(())
}
