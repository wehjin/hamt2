use crate::db::Attr;
use crate::db::reader::DbReader;
use crate::db::schema::Schema;
use crate::trie::Trie;
use crate::trie::base_storage::BaseStorageReadWrite;
pub mod cons;
pub mod query;
pub mod transact;

#[derive(Debug)]
pub struct Db<S: BaseStorageReadWrite> {
    pub(crate) schema: Schema,
    pub(crate) trie: Trie<S>,
}

impl<S: BaseStorageReadWrite> Db<S> {
    pub async fn to_reader(&self) -> DbReader<S::ReadOnly> {
        DbReader::load(self).await.expect("load reader")
    }
}

pub const QUERY: Attr = Attr("db/query");
pub const IDENT: Attr = Attr("db/ident");
pub const CARDINALITY: Attr = Attr("db/cardinality");
