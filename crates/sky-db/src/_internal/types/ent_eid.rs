use crate::Datom;
use crate::{Dat, Ein, Ent};
use std::collections::HashMap;
use std::ops::Index;

/// A map of temp entity names to their assigned `Ein`s, so a temp name refers
/// to the same entity everywhere in a transaction.
pub struct EntEid(HashMap<String, Ein>);

fn rand_nonce() -> [u8; 16] {
    let mut buf = [0u8; 16];
    getrandom::fill(&mut buf).expect("os rng");
    buf
}

impl EntEid {
    pub fn new(datoms: impl AsRef<[Datom]>) -> Self {
        let mut map = HashMap::new();
        for datom in datoms.as_ref() {
            if let Ent::Temp(name) = &datom.ent {
                map.entry(name.clone()).or_insert_with(|| Ein::user(&rand_nonce()));
            }
            if let Dat::Ent(Ent::Temp(name)) = &datom.dat {
                map.entry(name.clone()).or_insert_with(|| Ein::user(&rand_nonce()));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Attr, Dir, Val};

    fn datom(ent: Ent, dat: Dat) -> Datom {
        Datom {
            ent,
            attr: Attr::from("counter/count"),
            dat,
            dir: Dir::In,
        }
    }

    #[test]
    fn temp_names_in_ent_and_dat_get_distinct_eins() {
        let datoms = vec![datom(Ent::Temp("a".into()), Dat::Ent(Ent::Temp("b".into())))];
        let ent_eid = EntEid::new(&datoms);
        assert_ne!(ent_eid["a"], ent_eid["b"]);
    }

    #[test]
    fn distinct_temp_names_get_distinct_eins() {
        let datoms = vec![
            datom(Ent::Temp("a".into()), Dat::Val(Val::U32(1))),
            datom(Ent::Temp("b".into()), Dat::Val(Val::U32(2))),
        ];
        let ent_eid = EntEid::new(&datoms);
        assert_ne!(ent_eid["a"], ent_eid["b"]);
    }
}
