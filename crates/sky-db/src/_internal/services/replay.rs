use crate::_internal::db_trie;
use crate::trie::SkyTrie;
use crate::{Dat, Ein, Hash, Schema, Tx, linearize};
use std::collections::HashMap;

/// Replays a set of transactions into a materialized index.
///
/// Linearizes the transactions, then inserts each transaction's datoms in
/// order, resolving attribute idents against the running schema and folding
/// schema declarations as it goes. Returns the derived schema, the trie, the
/// linearized order (tx_ein -> position), and the tip hash.
pub async fn replay(
    txs: Vec<Tx>,
) -> anyhow::Result<(Schema, SkyTrie, HashMap<Ein, u32>, Option<Hash>)> {
    let map: HashMap<Hash, Tx> = txs.into_iter().map(|tx| (tx.hash(), tx)).collect();
    let order = linearize(&map);
    let mut schema = Schema::starter();
    let mut trie = SkyTrie::new();
    let mut order_map = HashMap::new();
    for (i, hash) in order.iter().enumerate() {
        let tx = &map[hash];
        let tx_ein = Ein::tx(&tx.nonce);
        order_map.insert(tx_ein, i as u32);
        trie.edit(async |trie| {
            for datom in &tx.datoms {
                let Some(aid) = schema.resolve(&datom.attr) else {
                    continue;
                };
                let val = match &datom.dat {
                    Dat::Val(val) => val.clone(),
                    Dat::Ent(_) => unreachable!("resolved datoms have no entity refs"),
                };
                db_trie::with_update(trie, aid, datom.ent.to_eid(), val, datom.dir, tx_ein).await?;
            }
            Ok(())
        })
        .await?;
        schema.apply(&tx.datoms);
    }
    let tip = order.last().copied();
    Ok((schema, trie, order_map, tip))
}
