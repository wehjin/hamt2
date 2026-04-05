use std::collections::HashMap;
use std::ops::Index;
use crate::db::component::MaxEid;
use crate::db::{Datom, Ein, Ent};

pub struct EntEid<'a>(HashMap<&'a str, Ein>);

impl EntEid<'_> {
    pub fn new(datoms: impl AsRef<[Datom]>, max_eid: &mut MaxEid) -> Self {
        let mut map = HashMap::new();
        let datoms = datoms.as_ref();
        for datom in datoms {
            match datom {
                Datom::Add(ent, _, _) => match ent {
                    Ent::Id(_) => (),
                    Ent::Temp(name) => {
                        let eid = max_eid.take(1).pop().expect("max_eid should exist");
                        map.insert(*name, eid);
                    }
                },
            }
        }
        Self(map)
    }
}

impl Index<&str> for EntEid<'_> {
    type Output = Ein;
    fn index(&self, name: &str) -> &Self::Output {
        &self.0[name]
    }
}