use crate::db::query::DbQuery;
use crate::db::{Attr, Db, Ein, Schema, Val};
use crate::space::Space;
use crate::space::mem::MemSpace;
use crate::trie::SpaceTrie;
use crate::{LoadError, QueryError};

#[derive(Debug)]
pub struct DbViewer<T: Space> {
    db: Db<T>,
}

impl<T: Space> DbViewer<T> {
    pub(crate) fn new(db: Db<T>) -> Self {
        DbViewer { db }
    }
}

impl DbViewer<MemSpace> {
    pub async fn load(space: MemSpace, attrs: impl AsRef<[Attr]>) -> Result<Self, LoadError> {
        let attrs = attrs.as_ref();
        let starter_db = Db {
            schema: Schema::starter(),
            trie: SpaceTrie::connect(&space).await?,
            space,
        };
        let db = Db {
            schema: Schema::load(attrs, &starter_db).await?,
            trie: starter_db.trie,
            space: starter_db.space,
        };
        let viewer = DbViewer { db };
        Ok(viewer)
    }
}

impl<T: Space> DbQuery for DbViewer<T> {
    fn find_val(
        &self,
        e: impl Into<Ein>,
        a: Attr,
    ) -> impl Future<Output = Result<Option<Val>, QueryError>> {
        self.db.find_val(e.into(), a)
    }
}
