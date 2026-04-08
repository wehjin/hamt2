use crate::db::schema::Schema;
use crate::db::Attr;
use crate::space::Space;
use crate::trie::SpaceTrie;
pub mod cons;
pub mod query;
pub mod transact;

#[derive(Debug)]
pub struct Db<T: Space> {
    pub(crate) schema: Schema,
    pub(crate) trie: SpaceTrie<T>,
    space: T,
}

pub const QUERY: Attr = Attr("db/query");
pub const IDENT: Attr = Attr("db/ident");
