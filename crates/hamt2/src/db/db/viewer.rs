use crate::db::query::DbQuery;
use crate::db::{Attr, Db, Ein, Schema, Val};
use crate::trie::base_storage::mem::MemBaseStorage;
use crate::trie::base_storage::BaseStorageReadWrite;
use crate::trie::Trie;
use crate::{LoadError, QueryError};

#[derive(Debug)]
pub struct DbViewer<S: BaseStorageReadWrite> {
    db: Db<S>,
}

impl<S: BaseStorageReadWrite> DbViewer<S> {
    pub(crate) fn new(db: Db<S>) -> Self {
        DbViewer { db }
    }
}

impl DbViewer<MemBaseStorage> {
    pub async fn load(storage: MemBaseStorage, attrs: impl AsRef<[Attr]>) -> Result<Self, LoadError> {
        let attrs = attrs.as_ref();
        let starter_db = Db {
            schema: Schema::starter(),
            trie: Trie::connect(storage).await?,
        };
        let db = Db {
            schema: Schema::load(attrs, &starter_db).await?,
            trie: starter_db.trie,
        };
        let viewer = DbViewer { db };
        Ok(viewer)
    }
}

impl<S: BaseStorageReadWrite + Clone> DbQuery for DbViewer<S> {
    fn find_val(
        &self,
        e: impl Into<Ein>,
        a: Attr,
    ) -> impl Future<Output = Result<Option<Val>, QueryError>> {
        self.db.find_val(e.into(), a)
    }
}
