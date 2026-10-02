use sky_db::Transact;
use sky_db::datom;
use sky_db::db_struct::Db;
use sky_db::find::{AllEins, AttrsOfEin, EinsWithAttr};
use sky_db::traits::DbQuery;
use sky_db::{Attr, ein, val};
use sky_types::storage::MemLoad;
use sky_types::storage::load::StoreLoad;

fn attr_count() -> Attr {
    Attr::from("counter/count")
}

#[tokio::test]
async fn db_reader_works() -> anyhow::Result<()> {
    let mut db = Db::new(MemLoad::new(), [attr_count()]).await?;
    db.transact([
        datom::add(1, attr_count(), val(10)),
        datom::add(2, attr_count(), val(20)),
        datom::add(3, attr_count(), val(30)),
    ])
    .await?;

    let mut eins = db.find(EinsWithAttr::new(attr_count())).await;
    eins.sort();
    assert_eq!(vec![ein(1), ein(2), ein(3)], eins);

    let reader = db.to_reader();
    assert_eq!(Some(val(10)), reader.find_val(1, attr_count()).await?);
    assert_eq!(Some(val(20)), reader.find_val(2, attr_count()).await?);
    assert_eq!(Some(val(30)), reader.find_val(3, attr_count()).await?);

    // The db stays usable after the reader is loaded.
    let db = db.transact([datom::add(4, attr_count(), val(40))]).await?;
    assert_eq!(Some(val(40)), db.find_val(4, attr_count()).await?);
    // The reader is a snapshot from before the new transact.
    assert_eq!(None, reader.find_val(4, attr_count()).await?);

    Ok(())
}

#[tokio::test]
async fn db_reader_finds_entities() {
    let mut db = Db::new(MemLoad::new(), [attr_count()]).await.unwrap();
    db.transact([datom::add(100, attr_count(), val(100))])
        .await
        .unwrap();

    let self1 = &db.to_reader();
    let mut eins = (async move { self1.find(AllEins).await }).await;
    eins.sort();
    assert_eq!(vec![ein(0), ein(1), ein(2), ein(100)], eins);
}

#[tokio::test]
async fn db_reader_lists_entity_attributes() {
    let mut db = Db::new(MemLoad::new(), [attr_count()]).await.unwrap();
    db.transact([datom::add(100, attr_count(), val(100))])
        .await
        .unwrap();
    let reader = db.to_reader();
    let find = AttrsOfEin::new(100);
    let attrs = (async move { reader.find(find).await }).await;
    assert_eq!(&[attr_count()], &attrs[..]);
}
