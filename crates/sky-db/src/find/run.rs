use crate::find::datalog::atom::{Atom, atom};
use crate::find::datalog::rule::rule;
use crate::find::datalog::term::term;
use crate::find::datalog::var::var;
use crate::find::datalog::Program;
use crate::db;
use crate::{FindResult, Pod};
use std::collections::HashMap;

pub(crate) fn run_find(
    pod: &Pod,
    select: impl Into<Vec<&'static str>>,
    where_: impl Into<Vec<Atom>>,
) -> FindResult {
    let select = select.into();
    let query_terms = select.iter().map(|s| term(var(*s))).collect::<Vec<_>>();
    let query_attr = db::query();
    let query_rule = rule(atom(query_attr.clone(), query_terms), where_.into());
    let program = Program::new([], [query_rule]);
    let kb = program.solve(pod.clone());
    let query_result = kb.query(query_attr);
    let mut found = FindResult::new();
    for row in query_result {
        let mut map = HashMap::new();
        if !row.is_empty() {
            let zipped = select
                .iter()
                .map(|s| s.to_string())
                .zip(row)
                .collect::<Vec<_>>();
            map.extend(zipped);
        }
        found.push(map);
    }
    found
}
