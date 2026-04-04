use crate::db::find::program::atom::Atom;
use crate::db::find::program::sub::Substitution;
use crate::db::find::program::term::Term;
use crate::db::{Attr, Val};
use std::collections::HashSet;

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

    pub fn unify_earth_atom(
        &self,
        earth_atom: &Atom,
        grounding_sub: &Substitution,
    ) -> Vec<Substitution> {
        let mut new_subs = Vec::new();
        for kb_atom in self.0.iter() {
            if let Some(extension) = earth_atom.unify(kb_atom) {
                let new_sub = grounding_sub.with_extension(extension);
                new_subs.push(new_sub);
            }
        }
        new_subs
    }
}
