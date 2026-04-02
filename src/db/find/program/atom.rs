use crate::db::find::program::kb::KnowledgeBase;
use crate::db::find::program::sub::Substitution;
use crate::db::find::program::term::Term;
use crate::db::find::program::var::Var;

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
            // Try improving the atom by replacing variables with values.
            let earth_atom = self.ground(&sub);
            // Try improving the substitution using facts from the KB.
            for kb_atom in kb.iter() {
                if let Some(tail_sub) = earth_atom.unify(kb_atom) {
                    let extended = sub.extend(tail_sub);
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
                Some(Substitution::new())
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
