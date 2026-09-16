use sky_db::db::Db;
use sky_db::db::attr_spec::AttrSpec;
use sky_db::db::cardinality::Cardinality;
use sky_db::find::ValsInSlot;
use sky_db::query::DbQuery;
use sky_trie::storage::mem::MemStorage;
use sky_types::db::Transact;
use sky_types::db::datom;
use sky_types::db::{Attr, val};

#[tokio::test]
async fn test_cardinality_one() -> anyhow::Result<()> {
    const COUNT: Attr = Attr("counter/count");
    let schema = [AttrSpec {
        attr: COUNT,
        cardinality: Cardinality::One,
    }];
    let mut db = Db::new(MemStorage::new(), schema).await?;
    db = db.transact([datom::add(100, COUNT, 100)]).await?;
    db = db.transact([datom::add(100, COUNT, 101)]).await?;
    db = db.transact([datom::add(100, COUNT, 102)]).await?;
    let vals = db.find(ValsInSlot::new(100, COUNT)).await;
    assert_eq!(vec![val(102)], vals);

    db = db.transact([datom::del(100, COUNT, 102)]).await?;
    let vals = db.find(ValsInSlot::new(100, COUNT)).await;
    assert!(vals.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_cardinality_many() -> anyhow::Result<()> {
    const COUNT: Attr = Attr("counter/count");
    let schema = [AttrSpec {
        attr: COUNT,
        cardinality: Cardinality::Many,
    }];
    let mut db = Db::new(MemStorage::new(), schema).await?;
    db = db.transact([datom::add(100, COUNT, 100)]).await?;
    db = db.transact([datom::add(100, COUNT, 101)]).await?;
    db = db.transact([datom::add(100, COUNT, 102)]).await?;
    let mut vals = db.find(ValsInSlot::new(100, COUNT)).await;
    vals.sort();
    assert_eq!(vec![val(100), val(101), val(102)], vals);

    db = db.transact([datom::del(100, COUNT, 101)]).await?;
    let mut vals = db.find(ValsInSlot::new(100, COUNT)).await;
    vals.sort();
    assert_eq!(vec![val(100), val(102)], vals);
    Ok(())
}
