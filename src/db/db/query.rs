use crate::db::component::val_table;
use crate::db::find::{Rule, ValsWithEntAttr};
use crate::db::component::key::{KEY_AEVT, KEY_MAX_TXID};
use crate::db::{Attr, Db, Ent, Txid, Val, Vid};
use crate::space::Space;
use crate::trie::mem::value::MemValue;
use crate::trie::SpaceTrie;
use crate::QueryError;
use async_stream::stream;
use futures::{pin_mut, StreamExt};

impl<T: Space> Db<T> {
    pub async fn max_tx(&self) -> Result<Txid, QueryError> {
        let Some(MemValue::U32(value)) = self.trie.query_value(KEY_MAX_TXID).await? else {
            panic!("max_tx not found");
        };
        Ok(Txid::from(value))
    }

    pub async fn find(&self, rule: &mut impl Rule) -> Result<bool, QueryError> {
        rule.update(&self.trie, &self.schema).await
    }

    pub async fn find_val(&self, e: Ent, a: Attr) -> Result<Option<Val>, QueryError> {
        let mut rule = ValsWithEntAttr::new("v", e, a);
        self.find(&mut rule).await?;
        let vals = rule.results("v");
        match vals.first() {
            None => Ok(None),
            Some(v) => Ok(Some(v.clone())),
        }
    }

    pub fn ev_stream(&self, a: Attr) -> impl futures::Stream<Item = (i32, Val)> {
        stream! {
            if let Some(evt_subtrie) = self.evt_subtrie(a).await {
                let evid_stream = evid_stream(evt_subtrie);
                pin_mut!(evid_stream);
                while let Some((eid, vid)) = evid_stream.next().await {
                    let val = val_table::query(&self.trie, Vid::from_id(vid)).await.ok().flatten().expect("val not found");
                    yield (eid, val);
                }
            }
        }
    }

    async fn evt_subtrie(&self, a: Attr) -> Option<SpaceTrie<T>> {
        let aid = self.schema[a].to_i32();
        let keys = [KEY_AEVT, aid];
        let evt_value = self.trie.deep_query_value(keys).await.ok().flatten();
        if let Some(evt) = evt_value {
            self.trie.to_subtrie_from_value(evt).await.ok()
        } else {
            None
        }
    }
}

fn evid_stream<T: Space>(
    evt_subtrie: SpaceTrie<T>,
) -> impl futures::Stream<Item = (i32, i32)> + use<T> {
    stream! {
        let evt_stream = evt_subtrie.subtrie_stream();
        pin_mut!(evt_stream);
        while let Some((eid, vt_trie)) = evt_stream.next().await {
            let vt_stream = vt_trie.u32_stream();
            pin_mut!(vt_stream);
            if let Some((vid, _)) = vt_stream.next().await {
                yield (eid, vid);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{dat, Datom};
    use crate::space::mem::MemSpace;
    #[tokio::test]
    async fn ev_stream_test() -> anyhow::Result<()> {
        const COUNT: Attr = Attr("counter", "count");
        let schema = vec![COUNT];
        let space = MemSpace::new();
        let mut db = Db::new(space, schema.clone()).await?;
        db = db
            .transact(vec![
                Datom::Add(Ent::from(10), COUNT, dat(Val::from(10))),
                Datom::Add(Ent::from(11), COUNT, dat(Val::from(11))),
            ])
            .await?;

        let ev_stream = db.ev_stream(COUNT);
        let mut ev_vec = ev_stream.collect::<Vec<_>>().await;
        ev_vec.sort_by_key(|ev| ev.0);
        assert_eq!(vec![(10, Val::from(10)), (11, Val::from(11))], ev_vec);
        Ok(())
    }
}
