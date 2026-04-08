use crate::db::component::db_trie;
use crate::db::component::key::KEY_MAX_TXID;
use crate::db::find::{EinAttrAny, Find};
use crate::db::{Attr, Db, Ein, Txid, Val};
use crate::space::Space;
use crate::trie::mem::value::MemValue;
use crate::QueryError;

impl<T: Space> Db<T> {
    pub async fn max_tx(&self) -> Result<Txid, QueryError> {
        let Some(MemValue::U32(value)) = self.trie.query_value(KEY_MAX_TXID).await? else {
            panic!("max_tx not found");
        };
        Ok(Txid::from(value))
    }

    pub async fn find_val(&self, e: Ein, a: Attr) -> Result<Option<Val>, QueryError> {
        let find = EinAttrAny::new(e, a);
        let vals = find.apply_db(self).await?;
        match vals.first() {
            None => Ok(None),
            Some(v) => Ok(Some(v.clone())),
        }
    }

    pub fn ev_stream(&self, a: Attr) -> impl futures::Stream<Item = (i32, Val)> {
        db_trie::ev_stream(&self.trie, a, &self.schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{dat, datom, ent};
    use crate::space::mem::MemSpace;
    use futures::StreamExt;
    #[tokio::test]
    async fn ev_stream_test() -> anyhow::Result<()> {
        const COUNT: Attr = Attr("counter/count");
        let schema = vec![COUNT];
        let space = MemSpace::new();
        let mut db = Db::new(space, schema.clone()).await?;
        db = db
            .transact(vec![
                datom(ent(10), COUNT, dat(Val::from(10))),
                datom(ent(11), COUNT, dat(Val::from(11))),
            ])
            .await?;

        let ev_stream = db.ev_stream(COUNT);
        let mut ev_vec = ev_stream.collect::<Vec<_>>().await;
        ev_vec.sort_by_key(|ev| ev.0);
        assert_eq!(vec![(10, Val::from(10)), (11, Val::from(11))], ev_vec);
        Ok(())
    }
}
