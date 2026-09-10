use crate::db::Attr;
use crate::db::schema::Schema;
use crate::space::Space;
use crate::trie::SpaceTrie;
use viewer::DbViewer;
pub mod cons;
pub mod query;
pub mod transact;
pub mod viewer;

#[derive(Debug)]
pub struct Db<T: Space> {
    pub(crate) schema: Schema,
    pub(crate) trie: SpaceTrie<T>,
    space: T,
}

impl<T: Space + Clone> Db<T> {
    pub fn to_viewer(&self) -> DbViewer<T> {
        let db = Db {
            schema: self.schema.clone(),
            trie: self.trie.clone(),
            space: self.space.clone(),
        };
        DbViewer::new(db)
    }
}

pub const QUERY: Attr = Attr("db/query");
pub const IDENT: Attr = Attr("db/ident");
pub const CARDINALITY: Attr = Attr("db/cardinality");
