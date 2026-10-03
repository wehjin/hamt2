use sky_db::Db;
use sky_db::DbQuery;
use sky_db::find::AllAttrs;
use sky_db::trie::SkyMap;
use sky_db::{Attr, attr};

fn attr_count() -> Attr {
    Attr::from("counter/count")
}

#[tokio::test]
async fn find_attrs_works() {
    let db = Db::new(SkyMap::new(), [attr_count()]).await.unwrap();
    let mut attrs = db.find(AllAttrs).await;
    attrs.sort();
    assert_eq!(
        attrs,
        vec![
            attr("counter/count"),
            attr("db/cardinality"),
            attr("db/ident")
        ]
    );
}
