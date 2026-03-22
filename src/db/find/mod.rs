use crate::db::{Attr, Ent};
use crate::hamt::trie::space::SpaceTrie;
use crate::QueryError;
use std::collections::HashMap;

mod ents_with_attr;
mod vals_with_ent_attr;

pub use ents_with_attr::*;
pub use vals_with_ent_attr::*;

pub trait Rule {
    type Output;
    fn results(&self, name: &'static str) -> &[Self::Output];
    fn update(&mut self, trie: &SpaceTrie, attrs: &HashMap<Attr, Ent>) -> Result<bool, QueryError>;
}
