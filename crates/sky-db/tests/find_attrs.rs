use sky_db::DbQuery;
use sky_db::create;
use sky_db::find::AllAttrs;
use sky_db::{Attr, attr};

fn attr_count() -> Attr {
    Attr::from("counter/count")
}

#[tokio::test]
async fn find_attrs_works() {
    let dir = tempfile::tempdir().unwrap();
    let pod = create(dir.path(), [attr_count()]).await.unwrap();
    let mut attrs = pod.find(AllAttrs).await;
    attrs.sort();
    assert_eq!(
        attrs,
        vec![
            attr("counter/count"),
            attr("db/cardinality"),
            attr("db/ident"),
            attr("tx/author"),
            attr("tx/instant"),
            attr("tx/nonce"),
            attr("tx/prev")
        ]
    );
}
