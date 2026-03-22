mod datom;
mod db;
mod txid;

pub use datom::*;
pub use db::*;
pub use txid::*;

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn transact_and_pull_works() {
        const ATTR_COUNT: Attr = Attr("counter/count");
        let db = Db::new(vec![ATTR_COUNT]).expect("new db");
        assert_eq!(Txid::FLOOR, db.max_tx().expect("max_tx"));

        let db = db
            .transact(vec![
                Datom::Add(Ent(15), ATTR_COUNT, Val::U32(15)),
                Datom::Add(Ent(5), ATTR_COUNT, Val::U32(5)),
            ])
            .expect("transact");
        assert_eq!(16, db.max_eid().expect("max_eid"));
        assert_eq!(Txid::FLOOR + 1, db.max_tx().expect("max_tx"));
        {
            let value15 = db.find_val(Ent(15), ATTR_COUNT).expect("find_val");
            let value5 = db.find_val(Ent(5), ATTR_COUNT).expect("find_val");
            assert_eq!(Some(Val::U32(15)), value15,);
            assert_eq!(Some(Val::U32(5)), value5);
        }
    }
}
