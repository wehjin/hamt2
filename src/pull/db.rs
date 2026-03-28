use crate::db::ent::Ent;
use crate::db::{Db, Eid};
use crate::pull::errors::BuildError;
use crate::pull::pull::Pull;
use crate::space::Space;

impl<T: Space> Db<T> {
    pub fn pull<U: Pull>(&self, id: Eid) -> Result<U::Target, BuildError> {
        let mut bindings = vec![];
        for attr in U::attrs() {
            let val = self.find_val(Ent::Id(id), attr)?;
            bindings.push((attr, val));
        }
        let target = U::build(bindings)?;
        Ok(target)
    }
}
