use sky_db::db::Db;
use sky_db::handle::DbHandle;
use sky_db::query::DbQuery;
use sky_trie::storage::mem::MemStorage;
use sky_types::db::datom;
use sky_types::db::{Attr, val};

fn attr_count() -> Attr {
    Attr::from("counter/count")
}

#[tokio::test]
async fn handle_works() -> anyhow::Result<()> {
    let db = Db::new(MemStorage::new(), [attr_count()]).await?;
    let handle = DbHandle::new(db).await;
    let handle2 = handle.clone();
    handle
        .transact([datom::add(1, attr_count(), val(10))])
        .await?;
    handle2
        .transact([datom::add(2, attr_count(), val(20))])
        .await?;

    // Confirm both transactions are effective in the handled db.
    let reader = handle.to_reader().await?;
    assert_eq!(Some(val(10)), reader.find_val(1, attr_count()).await?);
    assert_eq!(Some(val(20)), reader.find_val(2, attr_count()).await?);

    Ok(())
}
