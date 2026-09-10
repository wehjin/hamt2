use crate::QueryError;
use crate::db::query::DbQuery;
use crate::db::{Attr, Db, Ein, Val};
use crate::space::Space;

#[derive(Debug)]
pub struct DbViewer<T: Space> {
    db: Db<T>,
}

impl<T: Space> DbViewer<T> {
    pub(crate) fn new(state: Db<T>) -> Self {
        DbViewer { db: state }
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
