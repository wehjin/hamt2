use sky_db::db::Db;
use sky_db::find::EinsWithAttr;
use sky_db::query::DbQuery;
use sky_db::storage::MemDbStorage;
use sky_types::db::Transact;
use sky_types::db::datum;
use sky_types::db::{Attr, val};

const ATTR_COUNT: Attr = Attr("counter/count");

#[tokio::test]
async fn mem_db_works() -> anyhow::Result<()> {
    let db = Db::new(MemDbStorage::new(), [ATTR_COUNT]).await?;
    let db = db
        .transact([
            datum::add(1, ATTR_COUNT, val(10)),
            datum::add(2, ATTR_COUNT, val(20)),
            datum::add(3, ATTR_COUNT, val(30)),
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
