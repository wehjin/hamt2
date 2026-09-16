use crate::db::types::MaxEid;
use sky_types::db::Datom;
use sky_types::db::{Ein, Ent};
use std::collections::HashMap;
use std::ops::Index;

/// A map of temp entity names to their assigned `Ein`s, so a temp name refers
/// to the same entity everywhere in a transaction.
pub struct EntEid(HashMap<String, Ein>);

impl EntEid {
    pub fn new(datoms: impl AsRef<[Datom]>, max_eid: &mut MaxEid) -> Self {
        let mut map = HashMap::new();
        let datoms = datoms.as_ref();
        for datom in datoms {
            match &datom.ent {
                Ent::Id(_) => (),
                Ent::Temp(name) => {
                    let eid = max_eid.take(1).pop().expect("max_eid should exist");
                    map.insert(name.clone(), eid);
                }
            }
        }
        Self(map)
    }
}

impl Index<&str> for EntEid {
    type Output = Ein;
    fn index(&self, name: &str) -> &Self::Output {
        &self.0[name]
    }
}
