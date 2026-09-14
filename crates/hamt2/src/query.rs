use crate::QueryError;
use crate::db::db_trie;
use crate::db::types::key::KEY_MAX_TXID;
use crate::db::{Attr, Db, Ein, Txid, Val};
use crate::find::{Find, ValsInSlot};
use crate::trie::prelude::*;
use futures::FutureExt;

pub trait DbQuery {
    fn find<F: Find>(&self, find: F) -> impl Future<Output = Vec<F::Output>>;

    fn find_val(
        &self,
        e: impl Into<Ein>,
        a: Attr,
    ) -> impl Future<Output = Result<Option<Val>, QueryError>> {
        async move {
            let find = self.find(ValsInSlot::new(e, a)).await;
            Ok(find.first().cloned())
        }
    }

    fn get_val(&self, e: impl Into<Ein>, a: Attr) -> impl Future<Output = Val> {
        self.find_val(e.into(), a).map(|v| {
            v.expect("find_val should succeed")
                .expect("value should exist")
        })
    }
}

impl<S: ReadWriteTrieStorage> Db<S> {
    pub async fn max_tx(&self) -> Result<Txid, QueryError> {
        let Some(TrieValue::U32(value)) = self.trie.query_value(KEY_MAX_TXID).await? else {
            panic!("max_tx not found");
        };
        Ok(Txid::from(value))
    }

    pub fn ev_stream(&self, a: Attr) -> impl futures::Stream<Item = (i32, Val)> {
        db_trie::ev_stream(&self.trie, a, &self.schema)
    }
}

impl<S: ReadWriteTrieStorage> DbQuery for Db<S> {
    fn find<F: Find>(&self, find: F) -> impl Future<Output = Vec<F::Output>> {
        find.apply(&self.trie, &self.schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{dat, ent};
    use crate::types::datom;
    use futures::StreamExt;

    #[tokio::test]
    async fn ev_stream_test() -> anyhow::Result<()> {
        const COUNT: Attr = Attr("counter/count");
        let schema = vec![COUNT];
        let storage = MemTrieStorage::new();
        let mut db = Db::new(storage, schema.clone()).await?;
        db = db
            .transact(vec![
                datom::add(ent(10), COUNT, dat(Val::from(10))),
                datom::add(ent(11), COUNT, dat(Val::from(11))),
            ])
            .await?;

        let ev_stream = db.ev_stream(COUNT);
        let mut ev_vec = ev_stream.collect::<Vec<_>>().await;
        ev_vec.sort_by_key(|ev| ev.0);
        assert_eq!(vec![(10, Val::from(10)), (11, Val::from(11))], ev_vec);
        Ok(())
    }
}
