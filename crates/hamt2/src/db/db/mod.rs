use crate::db::reader::DbReader;
use crate::trie::prelude::*;
use crate::types::Attr;
use crate::types::schema::Schema;
pub mod cons;

#[derive(Debug)]
pub struct Db<S: ReadWriteTrieStorage> {
    pub(crate) schema: Schema,
    pub(crate) trie: Trie<S>,
}

impl<S: ReadWriteTrieStorage> Db<S> {
    pub async fn to_reader(&self) -> DbReader<S::ReadOnly> {
        DbReader::load(self).await.expect("load reader")
    }
}

pub const QUERY: Attr = Attr("db/query");
pub const IDENT: Attr = Attr("db/ident");
pub const CARDINALITY: Attr = Attr("db/cardinality");
