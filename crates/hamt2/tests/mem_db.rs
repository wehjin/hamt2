use hamt2::db::{Attr, Db, val};
use hamt2::find::EinsWithAttr;
use hamt2::query::DbQuery;
use hamt2::storage::MemDbStorage;
use hamt2::types::datom;

const ATTR_COUNT: Attr = Attr("counter/count");

#[tokio::test]
async fn mem_db_works() -> anyhow::Result<()> {
    let db = Db::new(MemDbStorage::new(), [ATTR_COUNT]).await?;
    let db = db
        .transact([
            datom::add(1, ATTR_COUNT, val(10)),
            datom::add(2, ATTR_COUNT, val(20)),
            datom::add(3, ATTR_COUNT, val(30)),
        ])
        .await?;

    let mut eins = db.find(EinsWithAttr::new(ATTR_COUNT)).await;
    eins.sort();

    for ein in eins {
        let count = db.find_val(ein, ATTR_COUNT).await?;
        println!("entity {:?} -> {:?}", ein, count);
    }

    Ok(())
}
