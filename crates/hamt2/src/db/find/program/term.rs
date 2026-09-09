use crate::db::find::program::var::Var;
use crate::db::Val;

pub fn term(from: impl Into<Term>) -> Term {
    from.into()
}

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

impl From<i32> for Term {
    fn from(i: i32) -> Self {
        Term::Val(Val::U32(i as u32))
    }
}

impl From<Val> for Term {
    fn from(v: Val) -> Self {
        Term::Val(v)
    }
}

impl From<Var> for Term {
    fn from(v: Var) -> Self {
        Term::Var(v)
    }
}
