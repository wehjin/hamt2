use crate::db::Attr;
use crate::db::schema::Schema;
use crate::trie::Trie;
use crate::trie::base_storage::BaseStorageReadWrite;
pub mod cons;
pub mod query;
pub mod reader;
pub mod transact;
pub mod viewer;

#[derive(Debug)]
pub struct Db<S: BaseStorageReadWrite> {
    pub(crate) schema: Schema,
    pub(crate) trie: Trie<S>,
}

pub const QUERY: Attr = Attr("db/query");
pub const IDENT: Attr = Attr("db/ident");
pub const CARDINALITY: Attr = Attr("db/cardinality");
