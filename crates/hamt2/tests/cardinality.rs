use hamt2::datom;
use hamt2::db::attr_spec::AttrSpec;
use hamt2::db::cardinality::Cardinality;
use hamt2::db::{Attr, Db, val};
use hamt2::find::ValsInSlot;
use hamt2::db_query::DbQuery;
use hamt2::trie::prelude::MemTrieStorage;

#[tokio::test]
async fn test_cardinality_one() -> anyhow::Result<()> {
    const COUNT: Attr = Attr("counter/count");
    let schema = [AttrSpec {
        attr: COUNT,
        cardinality: Cardinality::One,
    }];
    let mut db = Db::new(MemTrieStorage::new(), schema).await?;
    db = db.transact([datom::add(100, COUNT, 100)]).await?;
    db = db.transact([datom::add(100, COUNT, 101)]).await?;
    db = db.transact([datom::add(100, COUNT, 102)]).await?;
    let vals = db.find(ValsInSlot::new(100, COUNT)).await?;
    assert_eq!(vec![val(102)], vals);

    db = db.transact([datom::del(100, COUNT, 102)]).await?;
    let vals = db.find(ValsInSlot::new(100, COUNT)).await?;
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
    let mut db = Db::new(MemTrieStorage::new(), schema).await?;
    db = db.transact([datom::add(100, COUNT, 100)]).await?;
    db = db.transact([datom::add(100, COUNT, 101)]).await?;
    db = db.transact([datom::add(100, COUNT, 102)]).await?;
    let mut vals = db.find(ValsInSlot::new(100, COUNT)).await?;
    vals.sort();
    assert_eq!(vec![val(100), val(101), val(102)], vals);

    db = db.transact([datom::del(100, COUNT, 101)]).await?;
    let mut vals = db.find(ValsInSlot::new(100, COUNT)).await?;
    vals.sort();
    assert_eq!(vec![val(100), val(102)], vals);
    Ok(())
}
