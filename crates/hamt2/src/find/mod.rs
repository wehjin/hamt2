use crate::QueryError;

mod all_eins;
mod attr_with_name;
mod attrs_of_ein;
mod binds_for_attr;
mod eins_with_attr;
pub mod find_result;
mod vals_in_slot;

use crate::db::Schema;
use crate::db::component::db_trie;
use crate::db::datalog::atom::Atom;
use crate::trie::prelude::*;
pub use all_eins::*;
pub use attr_with_name::*;
pub use attrs_of_ein::*;
pub use binds_for_attr::*;
pub use eins_with_attr::*;
use find_result::FindResult;
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
        S: ReadTrieStorage,
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
    use crate::datom;
    use crate::db::{Attr, Db, ein, val};
    use crate::find::BindsForAttr;
    use crate::query::DbQuery;
    use crate::trie::prelude::*;

    #[tokio::test]
    async fn find_with_reader() {
        let attr = Attr::from("Counter/count");
        let store = MemTrieStorage::new();
        let db = Db::new(store, [attr]).await.unwrap();
        let txn = [datom::add(10, attr, 42)];
        let db = db.transact(txn).await.unwrap();
        let reader = db.to_reader().await;
        let found = reader.find(BindsForAttr::new(attr)).await.unwrap();
        assert_eq!(&[(ein(10), val(42))], found.as_slice());
    }
}
