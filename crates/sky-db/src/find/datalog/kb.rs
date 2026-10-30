use std::collections::HashSet;

use crate::{Attr, Fact, Pod, Val};

use super::binding::{Head, Pattern, Substitution};
use super::rule::Rule;
use super::term::Term;

/// A knowledge base: a pod snapshot plus the derived (rule-head) facts
/// accumulated during a fixpoint.
#[derive(Debug, Clone)]
pub struct KnowledgeBase {
    pub(crate) pod: Pod,
    facts: HashSet<Head>,
}

impl KnowledgeBase {
    pub fn new(pod: Pod) -> Self {
        Self {
            pod,
            facts: HashSet::new(),
        }
    }

    #[allow(dead_code)] // used by Program::solve
    pub fn query(&self, query: Attr) -> Vec<Vec<Val>> {
        let mut results = Vec::new();
        for head in &self.facts {
            if head.name == query {
                results.push(head.to_vals());
            }
        }
        results
    }

    /// Select-before-join: solve a body of patterns to a set of substitutions.
    pub fn derive_body_subs(&self, body: &[Pattern]) -> Vec<Substitution> {
        let mut subs = vec![Substitution::new()];
        for atom in body {
            let mut next = Vec::new();
            for sub in subs {
                let Some(earth) = atom.ground(&sub) else {
                    continue;
                };
                next.extend(self.unify_earth_atom(&earth, &sub));
            }
            subs = next;
        }
        subs
    }

    fn unify_earth_atom(&self, earth: &Pattern, sub: &Substitution) -> Vec<Substitution> {
        let mut out = Vec::new();
        for fact in self.facts_for_pattern(earth) {
            if let Some(ext) = earth.unify(&fact) {
                out.push(sub.with_extension(ext));
            }
        }
        out
    }

    fn facts_for_pattern(&self, p: &Pattern) -> Vec<Fact> {
        if let Term::Lit(aid) = &p.a {
            return self.pod.facts_for_attr(*aid);
        }
        if let Term::Lit(e) = &p.e {
            return self.pod.facts_for_entity(*e);
        }
        self.pod.all_facts()
    }

    #[allow(dead_code)] // used by Program::solve
    pub fn step(&self, rules: &[Rule]) -> Self {
        let mut new_facts = Vec::new();
        for rule in rules {
            new_facts.extend(rule.derive_facts(self));
        }
        let mut facts = self.facts.clone();
        facts.extend(new_facts);
        Self {
            pod: self.pod.clone(),
            facts,
        }
    }
}

impl PartialEq for KnowledgeBase {
    fn eq(&self, other: &Self) -> bool {
        self.facts == other.facts
    }
}

impl Eq for KnowledgeBase {}
