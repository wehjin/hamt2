use sky_db::DbQuery;
use sky_db::Pod;
use sky_db::Transact;
use sky_db::datom;
use sky_db::find::{AllEins, AttrsOfEin, BindsForAttr, EinsWithAttr};
use sky_db::{Attr, ein, val};

fn attr_count() -> Attr {
    Attr::from("counter/count")
}

#[tokio::test]
async fn clone_works() -> anyhow::Result<()> {
    let mut pod = Pod::new([attr_count()]).await?;
    pod.transact([
        datom::add(1, attr_count(), val(10)),
        datom::add(2, attr_count(), val(20)),
        datom::add(3, attr_count(), val(30)),
    ])
    .await?;

    let mut eins = pod.find(EinsWithAttr::new(attr_count())).await;
    eins.sort();
    assert_eq!(vec![ein(1), ein(2), ein(3)], eins);

    let reader = pod.clone();
    assert_eq!(Some(val(10)), reader.find_val(1, attr_count()).await?);
    assert_eq!(Some(val(20)), reader.find_val(2, attr_count()).await?);
    assert_eq!(Some(val(30)), reader.find_val(3, attr_count()).await?);

    // The pod stays usable after the reader is loaded.
    let pod = pod.transact([datom::add(4, attr_count(), val(40))]).await?;
    assert_eq!(Some(val(40)), pod.find_val(4, attr_count()).await?);
    // The reader is a snapshot from before the new transact.
    assert_eq!(None, reader.find_val(4, attr_count()).await?);

    Ok(())
}

#[tokio::test]
async fn clone_finds_binds() {
    let attr = Attr::from("Counter/count");
    let mut pod = Pod::new([attr.clone()]).await.unwrap();
    pod.transact([datom::add(10, attr.clone(), 42)])
        .await
        .unwrap();
    let reader = pod.clone();
    let found = reader.find(BindsForAttr::new(attr)).await;
    assert_eq!(&[(ein(10), val(42))], found.as_slice());
}

#[tokio::test]
async fn clone_finds_entities() {
    let mut pod = Pod::new([attr_count()]).await.unwrap();
    pod.transact([datom::add(100, attr_count(), val(100))])
        .await
        .unwrap();

    let self1 = pod.clone();
    let mut eins = (async move { self1.find(AllEins).await }).await;
    eins.sort();
    assert_eq!(vec![ein(0), ein(1), ein(2), ein(100)], eins);
}

#[tokio::test]
async fn clone_lists_entity_attributes() {
    let mut pod = Pod::new([attr_count()]).await.unwrap();
    pod.transact([datom::add(100, attr_count(), val(100))])
        .await
        .unwrap();
    let reader = pod.clone();
    let find = AttrsOfEin::new(100);
    let attrs = (async move { reader.find(find).await }).await;
    assert_eq!(&[attr_count()], &attrs[..]);
}
