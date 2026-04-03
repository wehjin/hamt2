use crate::db::schema::Schema;
use crate::space::Space;
use crate::trie::SpaceTrie;
pub mod cons;
pub mod query;
pub mod transact;

#[derive(Debug)]
pub struct Db<T: Space> {
    schema: Schema,
    trie: SpaceTrie<T>,
    space: T,
}
