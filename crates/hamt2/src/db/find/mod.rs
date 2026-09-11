use crate::QueryError;

mod any_attr_any;
mod any_attr_ignore;
mod ein_attr_any;
pub mod program;

use crate::db::Db;
use crate::db::component::db_trie;
use crate::db::find::program::atom::Atom;
use crate::db::find_result::FindResult;
use crate::db::schema::Schema;
use crate::trie::TrieQuery;
use crate::trie::base_storage::{BaseStorageRead, BaseStorageReadWrite};
pub use any_attr_any::*;
pub use any_attr_ignore::*;
pub use ein_attr_any::*;

pub trait Find {
    type Output;

    fn select(&self) -> Vec<&'static str>;
    fn where_(&self) -> Vec<Atom>;
    fn process(self, result: FindResult) -> Vec<Self::Output>;

    fn apply_db<S: BaseStorageReadWrite>(
        self,
        db: &Db<S>,
    ) -> impl Future<Output = Result<Vec<Self::Output>, QueryError>>
    where
        Self: Sized,
    {
        self.apply(&db.trie, &db.schema)
    }

    fn apply<T, S>(
        self,
        trie: &T,
        schema: &Schema,
    ) -> impl Future<Output = Result<Vec<Self::Output>, QueryError>>
    where
        Self: Sized,
        T: TrieQuery<S>,
        S: BaseStorageRead,
    {
        async move {
            let select = self.select();
            let where_ = self.where_();
            let result = db_trie::find(trie, schema, select, where_).await;
            let final_result = self.process(result);
            Ok(final_result)
        }
    }
}
