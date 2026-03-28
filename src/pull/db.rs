use crate::db::{Db, Eid};
use crate::pull::deserial::PullDeserializer;
use crate::pull::pull::Pull;
use crate::space::Space;
use crate::QueryError;

impl<T: Space> Db<T> {
    pub fn pull<'de, D: Pull<'de>>(&'de self, id: Eid) -> Result<D, QueryError> {
        let mut deserializer = PullDeserializer::new(id, self);
        let pulled = D::deserialize(&mut deserializer)?;
        Ok(pulled)
    }
}
