use crate::db::{Attr, Ent};
use crate::hamt::trie::space::SpaceTrie;
use crate::QueryError;
use std::collections::HashMap;

mod ents_with_attr;
mod vals_with_ent_attr;

use crate::space::Space;
pub use ents_with_attr::*;
pub use vals_with_ent_attr::*;

pub trait Rule {
    type Output;
    fn results(&self, name: &'static str) -> &[Self::Output];
    fn update<T: Space>(
        &mut self,
        trie: &SpaceTrie<T>,
        attrs: &HashMap<Attr, Ent>,
    ) -> Result<bool, QueryError>;
}
