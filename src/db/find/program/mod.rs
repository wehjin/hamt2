use crate::db::Val;
use std::collections::HashSet;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Var(pub &'static str);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Term {
    Var(Var),
    Val(Val),
}

impl Term {
    pub fn str_val(s: impl AsRef<str>) -> Self {
        Term::Val(Val::String(s.as_ref().to_string()))
    }
    pub fn var(s: &'static str) -> Self {
        Term::Var(Var(s))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Atom {
    pub pred_sym: &'static str,
    pub terms: Vec<Term>,
}
impl Atom {
    pub fn new(pred_sym: &'static str, terms: impl Into<Vec<Term>>) -> Self {
        let terms = terms.into();
        Self { pred_sym, terms }
    }
    pub fn to_vars(&self) -> Vec<Var> {
        self.terms
            .iter()
            .filter_map(|term| match term {
                Term::Var(var) => Some(*var),
                _ => None,
            })
            .collect()
    }
    pub fn ground(&self, substitution: &Substitution) -> Atom {
        let pred_sym = self.pred_sym;
        let mut terms = Vec::with_capacity(self.terms.len());
        {
            for term in self.terms.clone() {
                let term = match term {
                    existing @ Term::Val(_) => existing,
                    Term::Var(var) => match substitution.get(&var) {
                        Some(val) => Term::Val(val.clone()),
                        None => Term::Var(var),
                    },
                };
                terms.push(term);
            }
        }
        Atom { pred_sym, terms }
    }

    #[must_use]
    pub fn derive_subs(&self, subs: Vec<Substitution>, kb: &KnowledgeBase) -> Vec<Substitution> {
        let mut new_subs = Vec::new();
        for sub in subs {
            let earth_atom = self.ground(&sub);
            for kb_atom in &kb.0 {
                if let Some(unified_sub) = earth_atom.unify(kb_atom) {
                    let extended = sub.extend(unified_sub);
                    new_subs.push(extended);
                }
            }
        }
        new_subs
    }

    pub fn unify(&self, other: &Atom) -> Option<Substitution> {
        if self.pred_sym != other.pred_sym {
            return None;
        }
        debug_assert_eq!(self.terms.len(), other.terms.len());
        let candidates = self
            .terms
            .iter()
            .zip(other.terms.iter())
            .collect::<Vec<_>>();
        fn unify_terms(terms: &[(&Term, &Term)]) -> Option<Substitution> {
            if terms.len() == 0 {
                Some(Substitution::EMPTY)
            } else {
                let (a, b) = terms[0];
                match (a, b) {
                    (Term::Val(val_a), Term::Val(val_b)) => {
                        if val_a == val_b {
                            // Term is already unified, continue unifying the rest of the terms
                            unify_terms(&terms[1..])
                        } else {
                            // Conflict: different values
                            None
                        }
                    }
                    (Term::Var(var), Term::Val(val)) => {
                        let incomplete = unify_terms(&terms[1..])?;
                        match incomplete.get(&var) {
                            Some(tail_val) if tail_val != val => {
                                // Conflict: multiple values for the same variable. Can
                                // occur when the same variable is used in multiple terms.
                                None
                            }
                            _ => Some(incomplete.with_head(*var, val.clone())),
                        }
                    }
                    (_, Term::Var(_)) => unreachable!(
                        "unify_candidates should not be called with a variable on the right side"
                    ),
                }
            }
        }
        unify_terms(&candidates)
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub head: Atom,
    pub body: Vec<Atom>,
}
impl Rule {
    pub fn new(head: Atom, body: impl Into<Vec<Atom>>) -> Self {
        Self {
            head,
            body: body.into(),
        }
    }

    pub fn derive_facts(&self, kb: &KnowledgeBase) -> Vec<Atom> {
        let mut new_facts = Vec::new();
        for body_sub in self.derive_body_subs(kb) {
            let new_fact = self.head.ground(&body_sub);
            new_facts.push(new_fact);
        }
        new_facts
    }

    fn derive_body_subs(&self, kb: &KnowledgeBase) -> Vec<Substitution> {
        let mut body_subs = Vec::new();
        for atom in self.body.iter() {
            let atom_subs = atom.derive_subs(vec![Substitution::EMPTY], kb);
            body_subs.extend(atom_subs);
        }
        body_subs
    }

    pub fn is_range_restricted(&self) -> bool {
        let head_vars = self.head.to_vars();
        let body_vars = self
            .body
            .iter()
            .flat_map(|atom| atom.to_vars())
            .collect::<HashSet<_>>();
        for var in &head_vars {
            if !body_vars.contains(var) {
                return false;
            }
        }
        true
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
        let mut current = KnowledgeBase::empty().with_facts(self.facts);
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KnowledgeBase(HashSet<Atom>);
impl KnowledgeBase {
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

pub struct Substitution(Vec<(Var, Val)>);
impl Substitution {
    pub const EMPTY: Self = Self(vec![]);
    pub fn get(&self, var: &Var) -> Option<&Val> {
        for subst in &self.0 {
            if &subst.0 == var {
                return Some(&subst.1);
            }
        }
        None
    }
    pub fn with_head(mut self, var: Var, val: Val) -> Self {
        self.0.insert(0, (var, val));
        self
    }

    pub fn extend(&self, substitution: Substitution) -> Self {
        let mut pairs = self.0.clone();
        pairs.extend(substitution.0);
        Self(pairs)
    }
}
