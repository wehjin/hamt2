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

pub trait DbFinder {
    fn find<F: Find>(&self, find: F) -> impl Future<Output = Result<Vec<F::Output>, QueryError>>;
}

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

#[cfg(test)]
mod tests {
    use crate::db::find::{AnyAttrAny, DbFinder};
    use crate::db::{Attr, Db, datom, ein, val};
    use crate::trie::base_storage::mem::MemBaseStorage;

    #[tokio::test]
    async fn find_with_reader() {
        let attr = Attr::from("Counter/count");
        let store = MemBaseStorage::new();
        let db = Db::new(store, [attr]).await.unwrap();
        let txn = [datom::add(10, attr, 42)];
        let db = db.transact(txn).await.unwrap();
        let reader = db.to_reader().await;
        let found = reader.find(AnyAttrAny::new(attr)).await.unwrap();
        assert_eq!(&[(ein(10), val(42))], found.as_slice());
    }
}
