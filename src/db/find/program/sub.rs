use crate::db::find::program::var::Var;
use crate::db::Val;

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