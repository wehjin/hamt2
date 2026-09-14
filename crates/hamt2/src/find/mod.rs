mod all_eins;
mod attr_with_name;
mod attrs_of_ein;
mod binds_for_attr;
mod eins_with_attr;
pub mod types;
mod vals_in_slot;

use crate::crate_services::datalog::atom::Atom;

use crate::db::Schema;
use crate::db::db_trie;
use crate::trie::prelude::*;
pub use all_eins::*;
pub use attr_with_name::*;
pub use attrs_of_ein::*;
pub use binds_for_attr::*;
pub use eins_with_attr::*;
use sky_types::FindResult;
pub use vals_in_slot::*;

pub trait Find {
    type Output;

    fn select(&self) -> Vec<&'static str>;
    fn where_(&self) -> Vec<Atom>;
    fn process(self, result: FindResult) -> Vec<Self::Output>;

    fn apply<T, S>(self, trie: &T, schema: &Schema) -> impl Future<Output = Vec<Self::Output>>
    where
        Self: Sized,
        T: StorageTrieQuery<S>,
        S: SnapshotStorage,
    {
        async move {
            let select = self.select();
            let where_ = self.where_();
            let result = db_trie::find(trie, schema, select, where_).await;
            self.process(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::{Attr, Db, ein, val};
    use crate::find::BindsForAttr;
    use crate::query::DbQuery;
    use crate::trie::prelude::*;
    use crate::types::datom;

    #[tokio::test]
    async fn find_with_reader() {
        let attr = Attr::from("Counter/count");
        let store = MemTrieStorage::new();
        let db = Db::new(store, [attr]).await.unwrap();
        let txn = [datom::add(10, attr, 42)];
        let db = db.transact(txn).await.unwrap();
        let reader = db.to_reader().await;
        let found = reader.find(BindsForAttr::new(attr)).await;
        assert_eq!(&[(ein(10), val(42))], found.as_slice());
    }
}
