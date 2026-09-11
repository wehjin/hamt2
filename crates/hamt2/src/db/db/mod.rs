use crate::db::Attr;
use crate::db::schema::Schema;
use crate::trie::base_storage::BaseStorageReadWrite;
use crate::trie::Trie;
use viewer::DbViewer;
pub mod cons;
pub mod query;
pub mod transact;
pub mod viewer;

#[derive(Debug)]
pub struct Db<S: BaseStorageReadWrite> {
    pub(crate) schema: Schema,
    pub(crate) trie: Trie<S>,
}

impl<S: BaseStorageReadWrite + Clone> Db<S> {
    pub fn to_viewer(&self) -> DbViewer<S> {
        let db = Db {
            schema: self.schema.clone(),
            trie: self.trie.clone(),
        };
        DbViewer::new(db)
    }
}

pub const QUERY: Attr = Attr("db/query");
pub const IDENT: Attr = Attr("db/ident");
pub const CARDINALITY: Attr = Attr("db/cardinality");
