use sky_db::db::Db;
use sky_db::find::EinsWithAttr;
use sky_db::traits::DbQuery;
use sky_trie::storage::mem::MemStorage;
use sky_types::db::Transact;
use sky_types::db::datom;
use sky_types::db::{Attr, val};

fn attr_count() -> Attr {
    Attr::from("counter/count")
}

#[tokio::test]
async fn mem_db_works() -> anyhow::Result<()> {
    let db = Db::new(MemStorage::new(), [attr_count()]).await?;
    let db = db
        .transact([
            datom::add(1, attr_count(), val(10)),
            datom::add(2, attr_count(), val(20)),
            datom::add(3, attr_count(), val(30)),
        ])
        .await?;

    let mut eins = db.find(EinsWithAttr::new(attr_count())).await;
    eins.sort();

    for ein in eins {
        let count = db.find_val(ein, attr_count()).await?;
        println!("entity {:?} -> {:?}", ein, count);
    }

    Ok(())
}
