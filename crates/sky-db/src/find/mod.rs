mod all_eins;
mod attrs_of_ein;
mod binds_for_attr;
mod eins_with_attr;
pub mod types;
mod vals_in_slot;

pub use all_eins::*;
pub use attrs_of_ein::*;
pub use binds_for_attr::*;
pub use eins_with_attr::*;

pub use vals_in_slot::*;

#[cfg(test)]
mod tests {
    use crate::db::Db;
    use crate::find::BindsForAttr;
    use crate::traits::DbQuery;

    use sky_types::db::Transact;
    use sky_types::db::datom;
    use sky_types::db::{Attr, ein, val};
    use sky_types::storage::mem_edit_new;

    #[tokio::test]
    async fn find_with_reader() {
        let attr = Attr::from("Counter/count");
        let store = mem_edit_new();
        let db = Db::new(store, [attr.clone()]).await.unwrap();
        let txn = [datom::add(10, attr.clone(), 42)];
        let db = db.transact(txn).await.unwrap();
        let reader = db.to_reader();
        let found = reader.find(BindsForAttr::new(attr)).await;
        assert_eq!(&[(ein(10), val(42))], found.as_slice());
    }
}
