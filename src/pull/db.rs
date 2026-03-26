use crate::db::{Db, Ent};
use crate::pull::errors::BuildError;
use crate::pull::pull::Pull;
use crate::space::Space;

impl<T: Space> Db<T> {
    pub fn pull<U: Pull>(&self, id: i32) -> Result<U::Target, BuildError> {
        let ent = Ent(id);

        let mut bindings = vec![];
        for attr in U::attrs() {
            let val = self.find_val(ent, attr)?;
            bindings.push((attr, val));
        }
        let target = U::build(bindings)?;
        Ok(target)
    }
}