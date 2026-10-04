use crate::Pod;
use crate::_internal::datalog::atom::Atom;
use crate::_internal::datalog::rule::Rule;
use crate::_internal::datalog::sub::Substitution;
use crate::_internal::datalog::term::Term;
use crate::{Attr, Val};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct KnowledgeBase {
    pod: Pod,
    facts: HashSet<Atom>,
}

impl KnowledgeBase {
    pub fn from_facts(pod: Pod, facts: Vec<Atom>) -> Self {
        debug_assert!(facts.iter().all(|atom| atom.is_grounded()));
        Self {
            pod,
            facts: facts.into_iter().collect(),
        }
    }
    #[must_use]
    pub fn with_facts(&self, new_facts: Vec<Atom>) -> Self {
        debug_assert!(new_facts.iter().all(|atom| atom.is_grounded()));
        let mut facts = self.facts.clone();
        facts.extend(new_facts);
        Self {
            pod: self.pod.clone(),
            facts,
        }
    }

    #[must_use]
    pub fn query(&self, query: Attr) -> Vec<Vec<Val>> {
        let mut results = Vec::new();
        for atom in &self.facts {
            if atom.attr == query {
                let mut vals = Vec::new();
                for term in atom.terms.iter() {
                    let val = match term {
                        Term::Val(val) => val.clone(),
                        Term::Var(_var) => panic!("Ungrounded atom in kb: {:?}", atom),
                    };
                    vals.push(val);
                }
                results.push(vals);
            }
        }
        results
    }

    fn facts_stream<'a>(&'a self, earth_atom: &'a Atom) -> impl Iterator<Item = Atom> + 'a {
        let ev = (earth_atom.terms.len() == 2)
            .then(|| self.pod.ev_iter(earth_atom.attr.clone()))
            .into_iter()
            .flatten()
            .map(move |(e, v)| Atom::new(earth_atom.attr.clone(), [Term::from(e), Term::from(v)]));
        ev.chain(self.facts.iter().cloned())
    }

    pub fn unify_earth_atom(
        &self,
        earth_atom: &Atom,
        grounding_sub: &Substitution,
    ) -> Vec<Substitution> {
        let mut new_subs = Vec::new();
        for kb_atom in self.facts_stream(earth_atom) {
            if let Some(extension) = earth_atom.unify(&kb_atom) {
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
            debug_assert!(rule_facts.iter().all(|atom| atom.is_grounded()));
            new_facts.extend(rule_facts);
        }
        self.with_facts(new_facts)
    }
}

impl PartialEq for KnowledgeBase {
    fn eq(&self, other: &Self) -> bool {
        self.facts == other.facts
    }
}

impl Eq for KnowledgeBase {}
