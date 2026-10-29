use crate::Datom;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashMap;

/// A BLAKE3 content hash (the log entry key).
pub type Hash = [u8; 32];

/// A transaction: a hash-chain node plus its datoms.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Tx {
    pub prev: Option<Hash>,
    pub author: [u8; 32],
    pub nonce: [u8; 16],
    pub datoms: Vec<Datom>,
}

impl Tx {
    /// The content hash of this transaction (its stable identity).
    pub fn hash(&self) -> Hash {
        let bytes = postcard::to_stdvec(self).expect("serialize tx");
        *blake3::hash(&bytes).as_bytes()
    }
}

/// Linearize a set of transactions into a deterministic total order:
/// topological sort (causality via `prev`) with a hash tiebreak for forks.
pub fn linearize(txs: &HashMap<Hash, Tx>) -> Vec<Hash> {
    let mut children: HashMap<Option<Hash>, Vec<Hash>> = HashMap::new();
    for (hash, tx) in txs {
        children.entry(tx.prev).or_default().push(*hash);
    }
    let mut ready: BinaryHeap<Reverse<Hash>> = BinaryHeap::new();
    for root in children.get(&None).into_iter().flatten() {
        ready.push(Reverse(*root));
    }
    let mut order = Vec::with_capacity(txs.len());
    while let Some(Reverse(hash)) = ready.pop() {
        order.push(hash);
        if let Some(kids) = children.get(&Some(hash)) {
            for kid in kids {
                ready.push(Reverse(*kid));
            }
        }
    }
    debug_assert_eq!(order.len(), txs.len());
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(prev: Option<Hash>, nonce: u8, datoms: Vec<Datom>) -> Tx {
        Tx {
            prev,
            author: [0u8; 32],
            nonce: [nonce; 16],
            datoms,
        }
    }

    #[test]
    fn linearize_respects_causality_and_tiebreaks_forks() {
        // genesis -> a -> b, and genesis -> c (a fork off genesis)
        let genesis = tx(None, 1, vec![]);
        let g = genesis.hash();
        let a = tx(Some(g), 2, vec![]);
        let ha = a.hash();
        let b = tx(Some(ha), 3, vec![]);
        let hb = b.hash();
        let c = tx(Some(g), 4, vec![]);
        let hc = c.hash();

        let mut txs = HashMap::new();
        txs.insert(g, genesis);
        txs.insert(ha, a);
        txs.insert(hb, b);
        txs.insert(hc, c);

        let order = linearize(&txs);
        assert_eq!(4, order.len());
        // genesis is first; a before b; c is concurrent with a/b.
        assert_eq!(g, order[0]);
        let pos_a = order.iter().position(|h| *h == ha).unwrap();
        let pos_b = order.iter().position(|h| *h == hb).unwrap();
        assert!(pos_a < pos_b);
    }
}
