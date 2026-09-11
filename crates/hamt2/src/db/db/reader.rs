use crate::db::query::DbQuery;
use crate::db::{Attr, Ein, Schema, Val};
use crate::trie::base_storage::BaseStorageRead;
use crate::{LoadError, QueryError};

#[derive(Debug)]
pub struct DbReader<S: BaseStorageRead> {
    storage: S,
}

impl<S: BaseStorageRead> DbReader<S> {
    pub async fn load(storage: S, schema: Schema) -> Result<Self, LoadError> {
        let viewer = Self { storage };
        Ok(viewer)
    }
}

impl<S: BaseStorageRead> DbQuery for DbReader<S> {
    fn find_val(
        &self,
        e: impl Into<Ein>,
        a: Attr,
    ) -> impl Future<Output = Result<Option<Val>, QueryError>> {
        async move {
            todo!();
            Ok(None)
        }
    }
}
