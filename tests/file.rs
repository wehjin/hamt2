use crate::common::ATTR_GREETING;
use common::ATTR_COUNT;
use hamt2::db::Datom;
use hamt2::db::Db;
use hamt2::db::Ent;
use hamt2::db::Val;
use hamt2::space::file::FileSpace;

mod common;

#[tokio::test]
async fn file_db_works() {
    let file = tempfile::NamedTempFile::new().expect("tempfile");
    {
        let space = FileSpace::new(file.path()).expect("create file space");
        let db = Db::new(space, vec![ATTR_COUNT]).expect("new db");
        let db = db
            .transact(vec![Datom::Add(Ent::from(1), ATTR_COUNT, Val::U32(1))])
            .expect("transact");
        assert_eq!(
            Some(Val::U32(1)),
            db.find_val(Ent::from(1), ATTR_COUNT).expect("find_val")
        );
    }
    {
        let space = FileSpace::load(file.path()).expect("load red space");
        let db = Db::load(space, vec![ATTR_COUNT]).expect("load db");
        assert_eq!(
            Some(Val::U32(1)),
            db.find_val(Ent::from(1), ATTR_COUNT).expect("find_val")
        );
    }
}

#[tokio::test]
async fn file_db_strings_work() {
    let schema = vec![ATTR_GREETING];
    let file = tempfile::NamedTempFile::new().expect("tempfile");
    {
        let space = FileSpace::new(file.path()).expect("create file space");
        let db = Db::new(space, schema.clone()).expect("new db");
        let db = db
            .transact(vec![Datom::Add(
                Ent::from(1),
                ATTR_GREETING,
                Val::from("hello"),
            )])
            .expect("transact");
        assert_eq!(
            Some(Val::from("hello")),
            db.find_val(Ent::from(1), ATTR_GREETING).expect("find_val")
        );
    }
    {
        let space = FileSpace::load(file.path()).expect("load red space");
        let db = Db::load(space, schema).expect("load db");
        assert_eq!(
            Some(Val::from("hello")),
            db.find_val(Ent::from(1), ATTR_GREETING).expect("find_val")
        );
    }
}
