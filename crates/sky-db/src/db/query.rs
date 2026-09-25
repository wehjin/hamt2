use crate::db::db_trie;
use crate::db::types::key::KEY_MAX_TXID;
use crate::db::{Db, Txid};
use sky_types::trie::*;
use sky_types::db::QueryError;
use sky_types::db::{Attr, Val};
use sky_types::storage::BaseStore;

impl<S: BaseStore + Send + Sync> Db<S> {
    pub async fn max_tx(&self) -> Result<Txid, QueryError> {
        let Some(TrieValue::U32(value)) = self.trie.query(KEY_MAX_TXID).await? else {
            panic!("max_tx not found");
        };
        Ok(Txid::from(value))
    }

    pub fn ev_stream(&self, a: Attr) -> impl futures::Stream<Item = (i32, Val)> {
        db_trie::ev_stream(&self.trie, a, &self.schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use sky_types::db::Transact;
    use sky_types::db::datom;
    use sky_types::db::{dat, ent};
    use sky_types::storage::Mem;

    #[tokio::test]
    async fn ev_stream_test() -> anyhow::Result<()> {
        let count = || Attr::from("counter/count");
        let schema = vec![count()];
        let storage = Mem::new();
        let mut db = Db::new(storage, schema.clone()).await?;
        db = db
            .transact(vec![
                datom::add(ent(10), count(), dat(Val::from(10))),
                datom::add(ent(11), count(), dat(Val::from(11))),
            ])
            .await?;

        let ev_stream = db.ev_stream(count());
        let mut ev_vec = ev_stream.collect::<Vec<_>>().await;
        ev_vec.sort_by_key(|ev| ev.0);
        assert_eq!(vec![(10, Val::from(10)), (11, Val::from(11))], ev_vec);
        Ok(())
    }
}
