pub fn var(s: &'static str) -> Var {
    Var(s)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Var(pub &'static str);
