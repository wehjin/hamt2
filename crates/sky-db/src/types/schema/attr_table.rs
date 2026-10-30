use crate::attr_spec::AttrSpec;
use crate::attribute::AttributeDetails;
use crate::cardinality::Cardinality;
use crate::{Attr, Dat, Datom, Dir, Ein, Val, db, tx};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, Index};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttrTable {
    map: HashMap<Attr, AttributeDetails>,
    by_ein: HashMap<Ein, Attr>,
}

impl AttrTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            by_ein: HashMap::new(),
        }
    }

    pub fn to_attrs(&self) -> Vec<Attr> {
        self.map.keys().cloned().collect()
    }

    pub fn find_attr(&self, ein: Ein) -> Option<&Attr> {
        self.by_ein.get(&ein)
    }

    pub fn get(&self, attr: impl Into<Attr>) -> Option<AttributeDetails> {
        self.map.get(&attr.into()).cloned()
    }

    pub fn list(&self) -> Vec<AttributeDetails> {
        self.map.values().cloned().collect()
    }

    pub fn insert(&mut self, attribute: AttributeDetails) {
        self.by_ein.insert(attribute.ein, attribute.attr().clone());
        let key = attribute.attr().clone();
        self.map.insert(key, attribute);
    }
    pub fn extend(&mut self, attributes: impl IntoIterator<Item = AttributeDetails>) {
        for attribute in attributes {
            self.insert(attribute);
        }
    }

    /// Resolve an attribute ident to its ein.
    pub fn resolve(&self, ident: &Attr) -> Option<Ein> {
        self.map.get(ident).map(|d| d.ein)
    }

    /// Look up an attribute's cardinality by its ein.
    pub fn cardinality_of(&self, ein: Ein) -> Cardinality {
        self.map[&self.by_ein[&ein]].cardinality()
    }

    /// Fold schema declarations (`:db/ident` / `:db/cardinality` datoms) into
    /// the table. Only `Dir::In` datoms are applied; retractions are ignored.
    pub fn apply(&mut self, datoms: &[Datom]) {
        for d in datoms {
            if d.dir != Dir::In {
                continue;
            }
            let ein = d.ent.to_eid();
            if d.attr == db::ident() && let Dat::Val(Val::String(ident)) = &d.dat {
                let attr = Attr::from(ident.clone());
                self.map.entry(attr.clone()).or_insert_with(|| {
                    AttributeDetails::new(
                        ein,
                        AttrSpec {
                            attr: attr.clone(),
                            cardinality: Cardinality::One,
                        },
                    )
                });
                self.by_ein.insert(ein, attr);
            } else if d.attr == db::cardinality() && let Dat::Val(v) = &d.dat {
                let card = Cardinality::from(v.clone());
                if let Some(attr) = self.by_ein.get(&ein).cloned()
                    && let Some(details) = self.map.get_mut(&attr)
                {
                    details.spec.cardinality = card;
                }
            }
        }
    }

    pub fn starter() -> Self {
        let mut attr_table = Self::new();
        attr_table.extend(Self::starter_attributes());
        attr_table
    }

    pub(crate) fn starter_attributes() -> [AttributeDetails; 6] {
        [
            AttributeDetails::new(
                Ein::DB_IDENT,
                AttrSpec {
                    attr: db::ident(),
                    cardinality: Cardinality::One,
                },
            ),
            AttributeDetails::new(
                Ein::DB_CARDINALITY,
                AttrSpec {
                    attr: db::cardinality(),
                    cardinality: Cardinality::One,
                },
            ),
            AttributeDetails::new(
                Ein::attr(&tx::author()),
                AttrSpec {
                    attr: tx::author(),
                    cardinality: Cardinality::One,
                },
            ),
            AttributeDetails::new(
                Ein::attr(&tx::prev()),
                AttrSpec {
                    attr: tx::prev(),
                    cardinality: Cardinality::One,
                },
            ),
            AttributeDetails::new(
                Ein::attr(&tx::nonce()),
                AttrSpec {
                    attr: tx::nonce(),
                    cardinality: Cardinality::One,
                },
            ),
            AttributeDetails::new(
                Ein::attr(&tx::instant()),
                AttrSpec {
                    attr: tx::instant(),
                    cardinality: Cardinality::One,
                },
            ),
        ]
    }
}

impl Index<Attr> for AttrTable {
    type Output = AttributeDetails;
    fn index(&self, key: Attr) -> &Self::Output {
        &self.map[&key]
    }
}

impl Deref for AttrTable {
    type Target = HashMap<Attr, AttributeDetails>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{datom, val};

    #[test]
    fn apply_declares_ident_and_cardinality() {
        let mut table = AttrTable::starter();
        let ident = Attr::from("counter/count");
        let ein = Ein::attr(&ident);
        let datoms = vec![
            datom::add(ein, db::ident(), ident.as_ident()),
            datom::add(ein, db::cardinality(), val(1)), // Many
        ];
        table.apply(&datoms);
        assert_eq!(Some(ein), table.resolve(&ident));
        let details = table.get(ident.clone()).unwrap();
        assert_eq!(Cardinality::Many, details.cardinality());
    }
}
