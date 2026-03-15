use crate::base::Change;
use crate::client::{Client, Loader};
use crate::hamt::segment::Segment;
use crate::hamt::trie::mem::MemTrie;
use std::rc::Rc;

impl Client {
    pub fn transact(&mut self, changes: &[Change]) -> Result<(), TransactError> {
        let mut segment = Segment::new();
        let change_starts = segment.write_changes(changes);
        let mut trie = MemTrie::empty();
        for i in 0..change_starts.len() {
            match &changes[i] {
                Change::Deposit(entity, _attribute, _value) => {
                    trie = trie.insert(entity.0, change_starts[i] as u32);
                }
            }
        }
        let trie_start = segment.write_trie(&trie);
        let loader = Loader {
            segment: Some(Rc::new(segment)),
            root_start: Some(trie_start),
        };
        self.loader = Rc::new(loader);
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TransactError {}
