use hamt2::db::find::{AnyAttrIgnore, Find};
use hamt2::db::query::DbQuery;
use hamt2::db::{Attr, Db, datom, val};
use hamt2::trie::base_storage::mem::MemBaseStorage;

const ATTR_COUNT: Attr = Attr("counter/count");

#[tokio::test]
async fn mem_db_works() -> anyhow::Result<()> {
    let db = Db::new(MemBaseStorage::new(), [ATTR_COUNT]).await?;
    let db = db
        .transact([
            datom::add(1, ATTR_COUNT, val(10)),
            datom::add(2, ATTR_COUNT, val(20)),
            datom::add(3, ATTR_COUNT, val(30)),
        ])
        .await?;

    let mut eins = AnyAttrIgnore::new(ATTR_COUNT).apply_db(&db).await?;
    eins.sort();

    for ein in eins {
        let count = db.find_val(ein, ATTR_COUNT).await?;
        println!("entity {:?} -> {:?}", ein, count);
    }

    Ok(())
}
