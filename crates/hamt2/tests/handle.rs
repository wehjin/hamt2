use hamt2::db::handle::DbHandle;
use hamt2::db::{Attr, Db, datom, val};
use hamt2::trie::base_storage::mem::MemBaseStorage;

const ATTR_COUNT: Attr = Attr("counter/count");

#[tokio::test]
async fn handle_works() -> anyhow::Result<()> {
    let db = Db::new(MemBaseStorage::new(), [ATTR_COUNT]).await?;
    let handle = DbHandle::new(db).await;
    let handle2 = handle.clone();
    let _ = handle
        .transact([datom::add(1, ATTR_COUNT, val(10))])
        .await?;
    let _ = handle2
        .transact([datom::add(2, ATTR_COUNT, val(20))])
        .await?;
    Ok(())
}
