use crate::db::find::program::atom::Atom;
use crate::db::find::program::rule::Rule;
use crate::db::find::program::sub::Substitution;
use crate::db::find::program::term::Term;
use crate::db::{Attr, Db, Val};
use crate::space::Space;
use async_stream::stream;
use futures::{pin_mut, StreamExt};
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

    fn facts_stream(&self, earth_atom: &Atom) -> impl futures::Stream<Item = Atom> {
        stream! {
            if earth_atom.terms.len() == 2 {
                let ev_stream = self.db.ev_stream(earth_atom.attr);
                pin_mut!(ev_stream);
                while let Some((e,v)) = ev_stream.next().await {
                    yield Atom::new(earth_atom.attr, [Term::from(e), Term::from(v)]);
                }
            }
            for fact in self.facts.iter() {
                yield fact.clone();
            }
        }
    }

    pub async fn unify_earth_atom(
        &self,
        earth_atom: &Atom,
        grounding_sub: &Substitution,
    ) -> Vec<Substitution> {
        let mut new_subs = Vec::new();
        let facts_stream = self.facts_stream(earth_atom);
        pin_mut!(facts_stream);
        while let Some(kb_atom) = facts_stream.next().await {
            if let Some(extension) = earth_atom.unify(&kb_atom) {
                let new_sub = grounding_sub.with_extension(extension);
                new_subs.push(new_sub);
            }
        }
        new_subs
    }

    pub async fn step(&self, rules: &Vec<Rule>) -> Self {
        let mut new_facts = Vec::new();
        for rule in rules {
            let rule_facts = rule.derive_facts(self).await;
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
