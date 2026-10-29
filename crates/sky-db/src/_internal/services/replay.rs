use crate::_internal::db_trie;
use crate::trie::SkyTrie;
use crate::{Dat, Ein, Hash, Schema, TransactError, Tx, linearize};
use std::collections::HashMap;

/// The materialized index: the append-only trie plus the derived linearized order.
#[derive(Debug)]
pub struct Index {
    pub schema: Schema,
    pub trie: SkyTrie,
    pub order: HashMap<Ein, u32>,
    pub tip: Option<Hash>,
}

/// Applies a single transaction's datoms to the trie and folds its schema declarations.
pub(crate) async fn apply_tx(
    trie: &mut SkyTrie,
    schema: &mut Schema,
    tx: &Tx,
    tx_ein: Ein,
) -> Result<(), TransactError> {
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
    .await
    .map_err(TransactError::Rewind)?;
    schema.apply(&tx.datoms);
    Ok(())
}

/// Replays a set of transactions into a materialized index.
///
/// Linearizes the transactions, then inserts each transaction's datoms in
/// order, resolving attribute idents against the running schema and folding
/// schema declarations as it goes.
pub async fn replay(txs: Vec<Tx>) -> anyhow::Result<Index> {
    let map: HashMap<Hash, Tx> = txs.into_iter().map(|tx| (tx.hash(), tx)).collect();
    let order = linearize(&map);
    let mut schema = Schema::starter();
    let mut trie = SkyTrie::new();
    let mut order_map = HashMap::new();
    for (i, hash) in order.iter().enumerate() {
        let tx = &map[hash];
        let tx_ein = Ein::tx(&tx.nonce);
        order_map.insert(tx_ein, i as u32);
        apply_tx(&mut trie, &mut schema, tx, tx_ein).await?;
    }
    let tip = order.last().copied();
    Ok(Index {
        schema,
        trie,
        order: order_map,
        tip,
    })
}
