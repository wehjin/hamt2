use sky_db::DbQuery;
use sky_db::Transact;
use sky_db::create;
use sky_db::datom;
use sky_db::find::{AllEins, AttrsOfEin, BindsForAttr, EinsWithAttr};
use sky_db::{Attr, Ein, ein, val};

fn attr_count() -> Attr {
    Attr::from("counter/count")
}

#[tokio::test]
async fn clone_works() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let mut peer = create(dir.path(), [attr_count()]).await?;
    peer.transact([
        datom::add(1, attr_count(), val(10)),
        datom::add(2, attr_count(), val(20)),
        datom::add(3, attr_count(), val(30)),
    ])
    .await?;

    let pod = peer.snapshot().await;
    let mut eins = pod.find(EinsWithAttr::new(attr_count())).await;
    eins.sort();
    assert_eq!(vec![ein(1), ein(2), ein(3)], eins);

    let reader = peer.snapshot().await;
    assert_eq!(Some(val(10)), reader.find_val(1, attr_count()).await?);
    assert_eq!(Some(val(20)), reader.find_val(2, attr_count()).await?);
    assert_eq!(Some(val(30)), reader.find_val(3, attr_count()).await?);

    // The peer stays usable after the reader is loaded.
    peer.transact([datom::add(4, attr_count(), val(40))]).await?;
    let pod = peer.snapshot().await;
    assert_eq!(Some(val(40)), pod.find_val(4, attr_count()).await?);
    // The reader is a snapshot from before the new transact.
    assert_eq!(None, reader.find_val(4, attr_count()).await?);

    Ok(())
}

#[tokio::test]
async fn clone_finds_binds() {
    let attr = Attr::from("Counter/count");
    let dir = tempfile::tempdir().unwrap();
    let mut peer = create(dir.path(), [attr.clone()]).await.unwrap();
    peer.transact([datom::add(10, attr.clone(), 42)])
        .await
        .unwrap();
    let reader = peer.snapshot().await;
    let found = reader.find(BindsForAttr::new(attr)).await;
    assert_eq!(&[(ein(10), val(42))], found.as_slice());
}

#[tokio::test]
async fn clone_finds_entities() {
    let dir = tempfile::tempdir().unwrap();
    let mut peer = create(dir.path(), [attr_count()]).await.unwrap();
    peer.transact([datom::add(100, attr_count(), val(100))])
        .await
        .unwrap();

    let reader = peer.snapshot().await;
    let mut eins = reader.find(AllEins).await;
    eins.sort();
    // 1 user attr + 1 user entity + 2 tx entities (genesis + transact)
    assert_eq!(4, eins.len());
    assert!(eins.contains(&ein(100)));
    assert!(eins.contains(&Ein::attr(&attr_count())));
}

#[tokio::test]
async fn clone_lists_entity_attributes() {
    let dir = tempfile::tempdir().unwrap();
    let mut peer = create(dir.path(), [attr_count()]).await.unwrap();
    peer.transact([datom::add(100, attr_count(), val(100))])
        .await
        .unwrap();
    let reader = peer.snapshot().await;
    let find = AttrsOfEin::new(100);
    let attrs = reader.find(find).await;
    assert_eq!(&[attr_count()], &attrs[..]);
}
