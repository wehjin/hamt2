use crate::trie::space::trie::SpaceTrie;
use crate::QueryError;

mod ents_with_attr;
mod vals_with_ent_attr;

use crate::db::Schema;
use crate::space::Space;
pub use ents_with_attr::*;
pub use vals_with_ent_attr::*;

pub trait Rule {
    type Output;

    fn results(&self, name: &'static str) -> &[Self::Output];

    fn update<T: Space>(
        &mut self,
        trie: &SpaceTrie<T>,
        schema: &Schema,
    ) -> impl Future<Output = Result<bool, QueryError>>;
}
