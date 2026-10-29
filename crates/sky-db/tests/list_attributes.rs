use sky_db::cardinality::Cardinality;
use sky_db::create;
use sky_db::{Attribute};

#[tokio::test]
async fn list_attributes_works() {
    let dir = tempfile::tempdir().unwrap();
    let peer = create(dir.path(), ["counter/count"]).await.unwrap();
    let pod = peer.snapshot().await;
    let mut attributes = pod.list_attributes().await;
    attributes.sort();

    let mut idents = attributes.iter().map(Attribute::ident).collect::<Vec<_>>();
    idents.sort();
    assert_eq!(
        idents,
        vec![
            "counter/count",
            "db/cardinality",
            "db/ident",
            "tx/author",
            "tx/instant",
            "tx/nonce",
            "tx/prev"
        ]
    );

    let cards = attributes
        .iter()
        .map(Attribute::cardinality)
        .collect::<Vec<_>>();
    assert!(cards.iter().all(|c| *c == Cardinality::One));
}
