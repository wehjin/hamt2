use crate::base::{Attribute, Change, Entity, Txid, Value};
use crate::client::{Loader, Segment, SegmentIndex, SegmentTrie};
use crate::trie::key::TrieKey;
use std::rc::Rc;

pub enum Reader {
    Trie(SegmentTrie, Rc<Segment>),
    Empty,
}

impl Reader {
    pub fn new(loader: Rc<Loader>) -> Self {
        match (loader.segment.clone(), loader.root_start) {
            (Some(segment), Some(root_start)) => {
                let trie = segment.read_trie(SegmentIndex(root_start));
                Reader::Trie(trie, segment)
            }
            _ => Reader::Empty,
        }
    }

    pub fn top_txid(&self) -> Txid {
        Txid::FLOOR
    }

    pub fn query_value(&self, entity: Entity, _attribute: Attribute) -> Option<Value> {
        match self {
            Reader::Trie(trie, segment) => {
                let key = TrieKey::new(entity.0);
                match trie.query_value(key, segment.clone()) {
                    None => None,
                    Some(value_start) => {
                        let change_start = SegmentIndex(value_start as usize);
                        let (change, _start) = Change::from_be_bytes(change_start, &segment);
                        match change {
                            Change::Deposit(_, _, value) => Some(value),
                        }
                    }
                }
            }
            Reader::Empty => None,
        }
    }
}
