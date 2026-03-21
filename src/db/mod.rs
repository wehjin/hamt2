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
        let attr = Attr::from("count");
        let db = Db::new().expect("new db");
        {
            let max_tx = db.max_tx().expect("max_tx");
            assert_eq!(max_tx, Txid::FLOOR);
        }

        let db = db
            .transact(vec![Datom::Add(Ent(15), attr, Val::U32(15))])
            .expect("transact");
        {
            let max_tx = db.max_tx().expect("max_tx");
            let value = db.pull(Ent(15), attr).expect("pull");
            assert_eq!(value, Some(Val::U32(15)));
            assert_eq!(max_tx, Txid::FLOOR + 1);
        }
    }
}
