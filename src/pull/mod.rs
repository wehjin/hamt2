use crate::db::{Attr, Datom, Db, Ent, Val};
use crate::pull::errors::BuildError;
use crate::space::Space;

pub mod errors;
pub mod register;

#[cfg(test)]
mod tests;

pub trait Pull {
    type Target;
    fn attrs() -> Vec<Attr>;
    fn build(bindings: Vec<(Attr, Option<Val>)>) -> Result<Self::Target, BuildError>;
    fn to_datom(&self, id: i32) -> Vec<Datom>;
}

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
