use crate::{Attr, Ein, Val};
use super::var::Var;

/// A term in a pattern or fact: a variable, or a typed literal.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Term<T> {
    Var(Var),
    Lit(T),
}

impl<T> From<Var> for Term<T> {
    fn from(v: Var) -> Self {
        Term::Var(v)
    }
}

impl From<Ein> for Term<Ein> {
    fn from(e: Ein) -> Self {
        Term::Lit(e)
    }
}

impl From<Attr> for Term<Attr> {
    fn from(a: Attr) -> Self {
        Term::Lit(a)
    }
}

impl From<Val> for Term<Val> {
    fn from(v: Val) -> Self {
        Term::Lit(v)
    }
}

/// A value-position term (rule-head columns): coerces a `Var` or a `Val`.
#[allow(dead_code)] // used only when building rule heads
pub fn term(from: impl Into<Term<Val>>) -> Term<Val> {
    from.into()
}
