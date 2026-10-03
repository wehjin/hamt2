use sky_db;
use sky_db::cardinality::Cardinality;
use sky_db::{Attribute, Pod};

#[tokio::test]
async fn list_attributes_works() {
    let pod = Pod::new(["counter/count"]).await.unwrap();
    let mut attributes = pod.list_attributes().await;
    attributes.sort();

    let idents = attributes.iter().map(Attribute::ident).collect::<Vec<_>>();
    assert_eq!(idents, vec!["db/ident", "db/cardinality", "counter/count"]);

    let cards = attributes
        .iter()
        .map(Attribute::cardinality)
        .collect::<Vec<_>>();
    assert_eq!(
        cards,
        vec![Cardinality::One, Cardinality::One, Cardinality::One]
    );
}
