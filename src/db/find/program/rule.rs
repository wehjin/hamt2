use crate::db::find::program::atom::Atom;
use crate::db::find::program::kb::KnowledgeBase;
use crate::db::find::program::sub::Substitution;
use crate::space::Space;
use std::collections::HashSet;

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

    pub fn derive_facts<'a, T: Space>(&self, kb: &KnowledgeBase<'a, T>) -> Vec<Atom> {
        let mut new_facts = Vec::new();
        for body_sub in self.derive_body_subs(kb) {
            let new_fact = self.head.ground(&body_sub);
            new_facts.push(new_fact);
        }
        new_facts
    }

    fn derive_body_subs<'a, T: Space>(&self, kb: &KnowledgeBase<'a, T>) -> Vec<Substitution> {
        let mut body_subs = Vec::new();
        for atom in self.body.iter() {
            let body_atom_subs = atom.derive_body_atom_subs(vec![Substitution::new()], kb);
            body_subs.extend(body_atom_subs);
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
