use crate::db::Val;
use atom::Atom;
use rule::Rule;
use std::collections::HashSet;
use term::Term;
pub mod atom;
pub mod rule;
pub mod sub;
pub mod term;
pub mod var;

pub struct Program {
    facts: Vec<Atom>,
    rules: Vec<Rule>,
}

impl Program {
    pub fn new(facts: impl Into<Vec<Atom>>, rules: impl Into<Vec<Rule>>) -> Self {
        Self {
            facts: facts.into(),
            rules: rules.into(),
        }
    }

    pub fn solve(self) -> KnowledgeBase {
        for rule in &self.rules {
            if !rule.is_range_restricted() {
                panic!("The program is not range restricted: {:?}", rule);
            }
        }
        let mut current = KnowledgeBase::from_facts(self.facts);
        loop {
            let new = {
                let mut new_facts = Vec::new();
                for rule in &self.rules {
                    let rule_facts = rule.derive_facts(&current);
                    new_facts.extend(rule_facts);
                }
                current.with_facts(new_facts)
            };
            if new == current {
                return current;
            }
            current = new;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn program_test() {
        let program = Program::new(
            [
                Atom::new("advisor", [Term::str_val("Alice"), Term::str_val("Bob")]),
                Atom::new("advisor", [Term::str_val("Cliff"), Term::str_val("Bob")]),
            ],
            [Rule::new(
                Atom::new("query", [Term::var("x")]),
                [Atom::new("advisor", [Term::var("x"), Term::var("y")])],
            )],
        );
        let kb = program.solve();
        let answers = kb.query("query");
        dbg!(answers);
    }
}

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
    pub fn query(&self, query: &'static str) -> Vec<Vec<Val>> {
        let mut results = Vec::new();
        for atom in &self.0 {
            if atom.pred_sym == query {
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
