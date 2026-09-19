use crate::crate_services::datalog::atom::{Atom, atom};
use crate::crate_services::datalog::term::term;
use crate::crate_services::datalog::var::var;
use crate::traits::Find;
use sky_types::db;
use sky_types::db::FindResult;
use sky_types::db::schema::attr_spec::AttrSpec;
use sky_types::db::schema::attribute::Attribute;

pub struct SchemaLoader;

impl Find for SchemaLoader {
    type Output = Attribute;

    fn select(&self) -> Vec<&'static str> {
        vec!["ein", "ident", "cardinality"]
    }

    fn where_(&self) -> Vec<Atom> {
        vec![
            atom(db::ident(), [term(var("ein")), term(var("ident"))]),
            atom(
                db::cardinality(),
                [term(var("ein")), term(var("cardinality"))],
            ),
        ]
    }

    fn process(self, result: FindResult) -> Vec<Self::Output> {
        result
            .into_iter()
            .map(|map| {
                let ein = map.get("ein").cloned().unwrap();
                let ident = map.get("ident").unwrap().as_str();
                let cardinality = map.get("cardinality").cloned().unwrap();
                Attribute {
                    ein: ein.into(),
                    spec: AttrSpec {
                        attr: ident.into(),
                        cardinality: cardinality.into(),
                    },
                }
            })
            .collect::<Vec<_>>()
    }
}
