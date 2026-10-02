mod all_attrs;
mod all_eins;
mod attrs_of_ein;
mod binds_for_attr;
mod eins_with_attr;
mod fills_of_ein;
mod vals_in_slot;

pub mod types;

pub use all_attrs::*;
pub use all_eins::*;
pub use attrs_of_ein::*;
pub use binds_for_attr::*;
pub use eins_with_attr::*;
pub use fills_of_ein::*;
pub use vals_in_slot::*;

#[cfg(test)]
mod tests {
    use crate::Db;
    use crate::Transact;
    use crate::datom;
    use crate::find::BindsForAttr;
    use crate::traits::DbQuery;
    use crate::trie_storage::MemView;
    use crate::{Attr, ein, val};

    #[tokio::test]
    async fn find_with_reader() {
        let attr = Attr::from("Counter/count");
        let store = MemView::new();
        let mut db = Db::new(store, [attr.clone()]).await.unwrap();
        db.transact([datom::add(10, attr.clone(), 42)])
            .await
            .unwrap();
        let reader = db.to_reader();
        let found = reader.find(BindsForAttr::new(attr)).await;
        assert_eq!(&[(ein(10), val(42))], found.as_slice());
    }
}
