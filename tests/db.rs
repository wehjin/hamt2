use hamt2::db::attr::Attr;
use hamt2::db::ent::Ent;
use hamt2::db::find::{EntsWithAttr, Rule};
use hamt2::db::Val;
use hamt2::db::{Datom, Db, Txid};
use hamt2::space::file::FileSpace;
use hamt2::space::mem::MemSpace;
use hamt2::LoadError;

const ATTR_COUNT: Attr = Attr("counter", "count");
const ATTR_GREETING: Attr = Attr("speech", "greeting");

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
async fn load_works() {
    let space = Db::new(MemSpace::new(), vec![ATTR_COUNT])
        .expect("new db")
        .transact(vec![Datom::Add(Ent::from(1), ATTR_COUNT, Val::U32(1))])
        .expect("transact")
        .close();
    let db = Db::load(space, vec![ATTR_COUNT]).expect("load db");
    assert_eq!(
        Some(Val::U32(1)),
        db.find_val(Ent::from(1), ATTR_COUNT).expect("find_val")
    );
}

#[tokio::test]
async fn load_fails_with_unknown_attribute() {
    let space = Db::new(MemSpace::new(), vec![ATTR_COUNT])
        .expect("new db")
        .close();
    let result = Db::load(space, vec![ATTR_COUNT, ATTR_GREETING]);
    let Err(LoadError::UnknownAttr(ATTR_GREETING)) = result else {
        panic!("load should fail with unknown attr");
    };
}

#[tokio::test]
async fn transact_and_pull_simple() {
    let db = Db::new(MemSpace::new(), vec![ATTR_COUNT]).expect("new db");
    let db = db
        .transact(vec![Datom::Add(Ent::from(15), ATTR_COUNT, Val::U32(15))])
        .expect("transact");
    let v15 = db.find_val(Ent::from(15), ATTR_COUNT).expect("find_val");
    assert_eq!(Some(Val::U32(15)), v15);
}

#[tokio::test]
async fn entities_with_attr_works_for_single_entity() {
    let db = Db::new(MemSpace::new(), vec![ATTR_COUNT]).expect("new db");
    let db = db
        .transact(vec![Datom::Add(Ent::from(15), ATTR_COUNT, Val::U32(15))])
        .expect("transact");

    let mut rule = EntsWithAttr::new("e", ATTR_COUNT);
    db.find(&mut rule).expect("find");
    let ents = rule.results("e").to_vec();
    assert_eq!(vec![Ent::from(15)], ents);
}

#[tokio::test]
async fn entities_with_attr_works_for_two_entities() {
    let db = Db::new(MemSpace::new(), vec![ATTR_COUNT]).expect("new db");
    let db = db
        .transact(vec![
            Datom::Add(Ent::from(3), ATTR_COUNT, Val::U32(4)),
            Datom::Add(Ent::from(5), ATTR_COUNT, Val::U32(6)),
        ])
        .expect("transact");

    let mut rule = EntsWithAttr::new("e", ATTR_COUNT);
    db.find(&mut rule).expect("find");
    let mut ents = rule.results("e").to_vec();
    ents.sort();
    assert_eq!(vec![Ent::from(3), Ent::from(5)], ents);
}

#[test]
fn transact_assigns_id_to_ent() {
    let mut db = Db::new(MemSpace::new(), vec![ATTR_COUNT]).expect("new db");
    db = db
        .transact(vec![Datom::Add(
            Ent::Temp("new_count"),
            ATTR_COUNT,
            Val::U32(35),
        )])
        .expect("transact");
    db = db
        .transact(vec![Datom::Add(
            Ent::Temp("new_count"),
            ATTR_COUNT,
            Val::U32(35),
        )])
        .expect("transact");

    let mut rule = EntsWithAttr::new("e", ATTR_COUNT);
    db.find(&mut rule).expect("find");
    assert_eq!(2, rule.results("e").len());
}

#[tokio::test]
async fn transact_and_pull_works() {
    // Construct a new database.
    let db = Db::new(MemSpace::new(), vec![ATTR_COUNT]).expect("new db");
    assert_eq!(Txid::FLOOR, db.max_tx().expect("max_tx"));

    // Add a few datoms.
    let db = db
        .transact(vec![
            Datom::Add(Ent::from(15), ATTR_COUNT, Val::U32(15)),
            Datom::Add(Ent::from(5), ATTR_COUNT, Val::U32(5)),
        ])
        .expect("transact");

    let max_tx = db.max_tx().expect("max_tx");
    assert_eq!(Txid::FLOOR + 1, max_tx);

    let v15 = db.find_val(Ent::from(15), ATTR_COUNT).expect("find_val");
    assert_eq!(Some(Val::U32(15)), v15);

    let v5 = db.find_val(Ent::from(5), ATTR_COUNT).expect("find_val");
    assert_eq!(Some(Val::U32(5)), v5);

    // Discover the entities with an attribute.
    let mut rule = EntsWithAttr::new("e", ATTR_COUNT);
    db.find(&mut rule).expect("find");
    let mut ents = rule.results("e").to_vec();
    ents.sort();
    assert_eq!(vec![Ent::from(5), Ent::from(15)], ents);
}
