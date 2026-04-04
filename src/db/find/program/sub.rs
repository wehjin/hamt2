use crate::db::find::program::var::Var;
use crate::db::Val;
use std::collections::HashSet;

pub struct Substitution(HashSet<(Var, Val)>);

impl Substitution {
    pub fn new() -> Self {
        Self(HashSet::new())
    }
    pub fn get(&self, var: &Var) -> Option<&Val> {
        for (k, v) in &self.0 {
            if k == var {
                return Some(v);
            }
        }
        None
    }
    pub fn with_head(mut self, var: Var, val: Val) -> Self {
        self.0.insert((var, val));
        self
    }

    #[must_use]
    pub fn with_extension(&self, substitution: Substitution) -> Self {
        let mut pairs = self.0.clone();
        pairs.extend(substitution.0);
        Self(pairs)
    }
}
