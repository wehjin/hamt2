use crate::QueryError;

mod all_eins;
mod attrs_of_ein;
mod binds_for_attr;
mod eins_with_attr;
mod vals_in_slot;

use crate::db::Schema;
use crate::db::component::db_trie;
use crate::db::datalog::atom::Atom;
use crate::db::find_result::FindResult;
use crate::trie::TrieQuery;
use crate::trie::base_storage::BaseStorageRead;
pub use all_eins::*;
pub use attrs_of_ein::*;
pub use binds_for_attr::*;
pub use eins_with_attr::*;
pub use vals_in_slot::*;

pub trait Find {
    type Output;

    fn select(&self) -> Vec<&'static str>;
    fn where_(&self) -> Vec<Atom>;
    fn process(self, result: FindResult) -> Vec<Self::Output>;

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
    use crate::db::query::DbQuery;
    use crate::db::{Attr, Db, datom, ein, val};
    use crate::find::BindsForAttr;
    use crate::trie::base_storage::mem::MemBaseStorage;

    #[tokio::test]
    async fn find_with_reader() {
        let attr = Attr::from("Counter/count");
        let store = MemBaseStorage::new();
        let db = Db::new(store, [attr]).await.unwrap();
        let txn = [datom::add(10, attr, 42)];
        let db = db.transact(txn).await.unwrap();
        let reader = db.to_reader().await;
        let found = reader.find(BindsForAttr::new(attr)).await.unwrap();
        assert_eq!(&[(ein(10), val(42))], found.as_slice());
    }
}
