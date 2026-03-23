use hamt2::db::find::{EntsWithAttr, Rule};
use hamt2::db::{Attr, Datom, Db, Ent, Txid, Val};
use hamt2::space::mem::MemSpace;
use hamt2::LoadError;

const ATTR_COUNT: Attr = Attr("counter/count");
const ATTR_GREETING: Attr = Attr("speech/greeting");

#[tokio::test]
async fn load_works() {
    let space = Db::new(MemSpace::new(), vec![ATTR_COUNT])
        .expect("new db")
        .transact(vec![Datom::Add(Ent(1), ATTR_COUNT, Val::U32(1))])
        .expect("transact")
        .close();
    let db = Db::load(space, vec![ATTR_COUNT]).expect("load db");
    assert_eq!(
        Some(Val::U32(1)),
        db.find_val(Ent(1), ATTR_COUNT).expect("find_val")
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
async fn transact_and_pull_works() {
    // Construct a new database.
    let db = Db::new(MemSpace::new(), vec![ATTR_COUNT]).expect("new db");
    assert_eq!(Txid::FLOOR, db.max_tx().expect("max_tx"));

    // Add a few datoms.
    let db = db
        .transact(vec![
            Datom::Add(Ent(15), ATTR_COUNT, Val::U32(15)),
            Datom::Add(Ent(5), ATTR_COUNT, Val::U32(5)),
        ])
        .expect("transact");
    assert_eq!(16, db.max_eid().expect("max_eid"));
    assert_eq!(Txid::FLOOR + 1, db.max_tx().expect("max_tx"));
    assert_eq!(
        Some(Val::U32(15)),
        db.find_val(Ent(15), ATTR_COUNT).expect("find_val"),
    );
    assert_eq!(
        Some(Val::U32(5)),
        db.find_val(Ent(5), ATTR_COUNT).expect("find_val")
    );

    // Discover the entities with an attribute.
    let mut rule = EntsWithAttr::new("e", ATTR_COUNT);
    db.find(&mut rule).expect("find");
    let mut ents = rule.results("e").to_vec();
    ents.sort();
    assert_eq!(vec![Ent(5), Ent(15)], ents);
}
