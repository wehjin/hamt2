use crate::db::find::program::atom::Atom;
use crate::db::find::program::rule::Rule;
use crate::db::find::program::sub::Substitution;
use crate::db::find::program::term::Term;
use crate::db::{Attr, Db, Val};
use crate::space::Space;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct KnowledgeBase<'a, T: Space> {
    db: &'a Db<T>,
    facts: HashSet<Atom>,
}

impl<'a, T: Space> KnowledgeBase<'a, T> {
    pub fn from_facts(db: &'a Db<T>, facts: Vec<Atom>) -> Self {
        Self::empty(db).with_facts(facts)
    }
    pub fn empty(db: &'a Db<T>) -> Self {
        Self {
            db,
            facts: HashSet::new(),
        }
    }
    #[must_use]
    pub fn with_facts(&self, new_facts: Vec<Atom>) -> Self {
        let mut facts = self.facts.clone();
        facts.extend(new_facts);
        Self { db: self.db, facts }
    }

    #[must_use]
    pub fn query(&self, query: Attr) -> Vec<Vec<Val>> {
        let mut results = Vec::new();
        for atom in &self.facts {
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
        for kb_atom in self.facts.iter() {
            if let Some(extension) = earth_atom.unify(kb_atom) {
                let new_sub = grounding_sub.with_extension(extension);
                new_subs.push(new_sub);
            }
        }
        new_subs
    }

    pub fn step(&self, rules: &Vec<Rule>) -> Self {
        let mut new_facts = Vec::new();
        for rule in rules {
            let rule_facts = rule.derive_facts(self);
            new_facts.extend(rule_facts);
        }
        self.with_facts(new_facts)
    }
}

impl<'a, T: Space> PartialEq for KnowledgeBase<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.facts == other.facts
    }
}

impl<'a, T: Space> Eq for KnowledgeBase<'a, T> {}
