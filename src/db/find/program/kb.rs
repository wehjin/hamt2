use crate::db::find::program::atom::Atom;
use crate::db::find::program::term::Term;
use crate::db::{Attr, Val};
use std::collections::HashSet;
use std::ops::Deref;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KnowledgeBase(HashSet<Atom>);

impl KnowledgeBase {
    pub fn from_facts(facts: Vec<Atom>) -> Self {
        Self::empty().with_facts(facts)
    }
    pub fn empty() -> Self {
        Self(HashSet::new())
    }
    #[must_use]
    pub fn with_facts(&self, new_facts: Vec<Atom>) -> Self {
        let mut facts = self.0.clone();
        facts.extend(new_facts);
        Self(facts)
    }

    #[must_use]
    pub fn query(&self, query: Attr) -> Vec<Vec<Val>> {
        let mut results = Vec::new();
        for atom in &self.0 {
            if atom.attr == query {
                let mut vals = Vec::new();
                for term in atom.terms.iter() {
                    if let Term::Val(val) = term {
                        vals.push(val.clone());
                    }
                }
                results.push(vals);
            }
        }
        results
    }
}

impl Deref for KnowledgeBase {
    type Target = HashSet<Atom>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
