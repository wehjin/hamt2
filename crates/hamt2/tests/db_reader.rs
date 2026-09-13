use hamt2::datom;
use hamt2::db::reader::DbReader;
use hamt2::db::{Attr, Db, ein, val};
use hamt2::find::{AllEins, AttrsOfEin, EinsWithAttr};
use hamt2::db_query::DbQuery;
use hamt2::trie::prelude::MemTrieStorage;

const ATTR_COUNT: Attr = Attr("counter/count");

#[tokio::test]
async fn db_reader_works() -> anyhow::Result<()> {
    let db = Db::new(MemTrieStorage::new(), [ATTR_COUNT]).await?;
    let db = db
        .transact([
            datom::add(1, ATTR_COUNT, val(10)),
            datom::add(2, ATTR_COUNT, val(20)),
            datom::add(3, ATTR_COUNT, val(30)),
        ])
        .await?;

    let mut eins = db.find(EinsWithAttr::new(ATTR_COUNT)).await?;
    eins.sort();
    assert_eq!(vec![ein(1), ein(2), ein(3)], eins);

    let reader = DbReader::load(&db).await?;
    assert_eq!(Some(val(10)), reader.find_val(1, ATTR_COUNT).await?);
    assert_eq!(Some(val(20)), reader.find_val(2, ATTR_COUNT).await?);
    assert_eq!(Some(val(30)), reader.find_val(3, ATTR_COUNT).await?);

    // The db stays usable after the reader is loaded.
    let db = db.transact([datom::add(4, ATTR_COUNT, val(40))]).await?;
    assert_eq!(Some(val(40)), db.find_val(4, ATTR_COUNT).await?);
    // The reader is a snapshot from before the new transact.
    assert_eq!(None, reader.find_val(4, ATTR_COUNT).await?);

    Ok(())
}

#[tokio::test]
async fn db_reader_finds_entities() {
    let db = Db::new(MemTrieStorage::new(), [ATTR_COUNT])
        .await
        .unwrap()
        .transact([datom::add(100, ATTR_COUNT, val(100))])
        .await
        .unwrap();

    let mut eins = db.to_reader().await.get(AllEins).await;
    eins.sort();
    assert_eq!(vec![ein(0), ein(1), ein(2), ein(100)], eins);
}

#[tokio::test]
async fn db_reader_lists_entity_attributes() {
    let db = Db::new(MemTrieStorage::new(), [ATTR_COUNT])
        .await
        .unwrap()
        .transact([datom::add(100, ATTR_COUNT, val(100))])
        .await
        .unwrap();
    let reader = db.to_reader().await;
    let attrs = reader.get(AttrsOfEin::new(100)).await;
    assert_eq!(&[ATTR_COUNT], &attrs[..]);
}
