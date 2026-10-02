use sky_db::db_struct::Db;
use sky_db::find::AllAttrs;
use sky_db::traits::DbQuery;
use sky_db::{Attr, attr};
use sky_types::storage::MemLoad;
use sky_types::storage::load::StoreLoad;

fn attr_count() -> Attr {
    Attr::from("counter/count")
}

#[tokio::test]
async fn find_attrs_works() {
    let db = Db::new(MemLoad::new(), [attr_count()]).await.unwrap();
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
