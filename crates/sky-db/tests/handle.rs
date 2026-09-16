use sky_db::db::Db;
use sky_db::handle::DbHandle;
use sky_db::query::DbQuery;
use sky_db::storage::MemDbStorage;
use sky_types::db::datom;
use sky_types::db::{Attr, val};

const ATTR_COUNT: Attr = Attr("counter/count");

#[tokio::test]
async fn handle_works() -> anyhow::Result<()> {
    let db = Db::new(MemDbStorage::new(), [ATTR_COUNT]).await?;
    let handle = DbHandle::new(db).await;
    let handle2 = handle.clone();
    handle
        .transact([datom::add(1, ATTR_COUNT, val(10))])
        .await?;
    handle2
        .transact([datom::add(2, ATTR_COUNT, val(20))])
        .await?;

    // Confirm both transactions are effective in the handled db.
    let reader = handle.to_reader().await?;
    assert_eq!(Some(val(10)), reader.find_val(1, ATTR_COUNT).await?);
    assert_eq!(Some(val(20)), reader.find_val(2, ATTR_COUNT).await?);

    Ok(())
}
