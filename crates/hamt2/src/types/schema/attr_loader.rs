use crate::crate_services::datalog::atom::{Atom, atom};
use crate::crate_services::datalog::term::term;
use crate::crate_services::datalog::var::var;
use crate::db;
use crate::db::Attr;
use crate::db::attr_spec::AttrSpec;
use crate::db::attribute::Attribute;
use crate::find::Find;
use crate::find::types::find_result::FindResult;
use std::collections::HashMap;

pub struct AttributeLoader(HashMap<String, Attr>);
impl AttributeLoader {
    pub fn new(attrs: impl AsRef<[Attr]>) -> Self {
        Self(
            attrs
                .as_ref()
                .into_iter()
                .map(|attr| (attr.as_ident().to_string(), attr.clone()))
                .collect(),
        )
    }
}
impl Find for AttributeLoader {
    type Output = Attribute;

    fn select(&self) -> Vec<&'static str> {
        vec!["ein", "ident", "cardinality"]
    }

    fn where_(&self) -> Vec<Atom> {
        vec![
            atom(db::IDENT, [term(var("ein")), term(var("ident"))]),
            atom(
                db::CARDINALITY,
                [term(var("ein")), term(var("cardinality"))],
            ),
        ]
    }

    fn process(self, result: FindResult) -> Vec<Self::Output> {
        result
            .into_iter()
            .filter_map(|map| {
                let ein = map.get("ein").cloned().unwrap();
                let ident = map.get("ident").unwrap().as_str();
                let cardinality = map.get("cardinality").cloned().unwrap();
                if let Some(attr) = self.0.get(ident) {
                    let attribute = Attribute {
                        ein: ein.into(),
                        spec: AttrSpec {
                            attr: attr.clone(),
                            cardinality: cardinality.into(),
                        },
                    };
                    Some(attribute)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }
}
