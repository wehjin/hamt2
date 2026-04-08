use hamt2::db::Ent;
use hamt2::db::Val;
use hamt2::db::{dat, ein, Db};
use hamt2::db::{Attr, Datom};
use hamt2::space::file::FileSpace;

pub const ATTR_COUNT: Attr = Attr("counter/count");
pub const ATTR_GREETING: Attr = Attr("speech/greeting");

#[tokio::test]
async fn file_db_works() {
    let file = tempfile::NamedTempFile::new().expect("tempfile");
    {
        let space = FileSpace::new(&file).await.expect("create file space");
        let db = Db::new(space, vec![ATTR_COUNT]).await.expect("new db");
        let db = db
            .transact(vec![Datom::Add(Ent::from(1), ATTR_COUNT, dat(Val::U32(1)))])
            .await
            .expect("transact");
        assert_eq!(
            Some(Val::U32(1)),
            db.find_val(ein(1), ATTR_COUNT).await.expect("find_val")
        );
    }
    {
        let space = FileSpace::load(&file).await.expect("load red space");
        let db = Db::load(space, vec![ATTR_COUNT]).await.expect("load db");
        assert_eq!(
            Some(Val::U32(1)),
            db.find_val(ein(1), ATTR_COUNT).await.expect("find_val")
        );
    }
}

#[tokio::test]
async fn file_db_strings_work() {
    let schema = vec![ATTR_GREETING];
    let file = tempfile::NamedTempFile::new().expect("tempfile");
    {
        let space = FileSpace::new(&file).await.expect("create file space");
        let db = Db::new(space, schema.clone()).await.expect("new db");
        let db = db
            .transact(vec![Datom::Add(
                Ent::from(1),
                ATTR_GREETING,
                dat(Val::from("hello")),
            )])
            .await
            .expect("transact");
        assert_eq!(
            Some(Val::from("hello")),
            db.find_val(ein(1), ATTR_GREETING).await.expect("find_val")
        );
    }
    {
        let space = FileSpace::load(&file).await.expect("load red space");
        let db = Db::load(space, schema).await.expect("load db");
        assert_eq!(
            Some(Val::from("hello")),
            db.find_val(ein(1), ATTR_GREETING).await.expect("find_val")
        );
    }
}
