use std::collections::HashMap;

use crate::{Attr, AttrEin, Ein, Schema, Val};

use super::term::Term;
use super::var::Var;

/// A canonical, typed binding for a variable.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Binding {
    Ein(Ein),
    Attr(AttrEin),
    Val(Val),
}

/// A query pattern whose attribute is still an unresolved ident.
#[derive(Debug, Clone)]
pub struct RawPattern {
    pub e: Term<Ein>,
    pub a: Term<Attr>,
    pub v: Term<Val>,
}

/// A query pattern whose attribute is resolved to its `AttrEin`.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub e: Term<Ein>,
    pub a: Term<AttrEin>,
    pub v: Term<Val>,
}

/// An erased, arity-N derived relation head (a rule head / query result row).
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Head {
    pub name: Attr,
    pub terms: Vec<Term<Val>>,
}

pub fn pattern(
    e: impl Into<Term<Ein>>,
    a: impl Into<Term<Attr>>,
    v: impl Into<Term<Val>>,
) -> RawPattern {
    RawPattern {
        e: e.into(),
        a: a.into(),
        v: v.into(),
    }
}

#[allow(dead_code)] // rule heads are exercised only by program_test
pub fn head(name: impl Into<Attr>, terms: impl Into<Vec<Term<Val>>>) -> Head {
    Head {
        name: name.into(),
        terms: terms.into(),
    }
}

/// Resolve a raw pattern's attribute ident to its `AttrEin`.
pub fn resolve(p: RawPattern, schema: &Schema) -> Option<Pattern> {
    Some(Pattern {
        e: p.e,
        v: p.v,
        a: match p.a {
            Term::Lit(ident) => Term::Lit(schema.resolve(&ident)?),
            Term::Var(v) => Term::Var(v),
        },
    })
}

impl RawPattern {
    pub fn vars(&self) -> Vec<Var> {
        let mut vs = Vec::new();
        if let Term::Var(v) = &self.e {
            vs.push(*v);
        }
        if let Term::Var(v) = &self.a {
            vs.push(*v);
        }
        if let Term::Var(v) = &self.v {
            vs.push(*v);
        }
        vs
    }
}

impl Head {
    #[allow(dead_code)]
    pub fn vars(&self) -> Vec<Var> {
        let mut vs = Vec::new();
        for t in &self.terms {
            if let Term::Var(v) = t {
                vs.push(*v);
            }
        }
        vs
    }

    pub fn ground(&self, sub: &Substitution) -> Head {
        Head {
            name: self.name.clone(),
            terms: self
                .terms
                .iter()
                .map(|t| match t {
                    Term::Lit(v) => Term::Lit(v.clone()),
                    Term::Var(v) => sub
                        .get(v)
                        .map(|b| Term::Lit(b.to_val()))
                        .unwrap_or(Term::Var(*v)),
                })
                .collect(),
        }
    }

    pub fn is_grounded(&self) -> bool {
        self.terms.iter().all(|t| matches!(t, Term::Lit(_)))
    }

    pub fn to_vals(&self) -> Vec<Val> {
        self.terms
            .iter()
            .map(|t| match t {
                Term::Lit(v) => v.clone(),
                Term::Var(_) => unreachable!("head should be grounded"),
            })
            .collect()
    }
}

impl Binding {
    pub fn as_ein(&self) -> Option<Ein> {
        match self {
            Binding::Ein(e) => Some(*e),
            Binding::Attr(a) => Some(a.ein()),
            Binding::Val(Val::Ein(e)) => Some(*e),
            // Interim lenient: a U32 value derefs to an ein (delete at the flip).
            Binding::Val(Val::U32(n)) => Some(Ein(*n as i32)),
            _ => None,
        }
    }

    pub fn as_attr(&self) -> Option<AttrEin> {
        match self {
            Binding::Attr(a) => Some(*a),
            _ => None,
        }
    }

    pub fn to_val(&self) -> Val {
        match self {
            // Interim: render eins/attrs as U32 (delete at the flip).
            Binding::Ein(e) => Val::U32(e.to_i32() as u32),
            Binding::Attr(a) => Val::U32(a.to_i32() as u32),
            Binding::Val(v) => v.clone(),
        }
    }
}

/// A substitution: variable name -> canonical typed binding.
#[derive(Debug, Clone, Default)]
pub struct Substitution(HashMap<Var, Binding>);

impl Substitution {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&self, v: &Var) -> Option<&Binding> {
        self.0.get(v)
    }

    pub fn bind(&mut self, v: Var, b: Binding) -> Option<()> {
        match self.0.get(&v) {
            None => {
                self.0.insert(v, b);
                Some(())
            }
            Some(existing) => {
                let canonical = unify_bindings(existing, &b)?;
                self.0.insert(v, canonical);
                Some(())
            }
        }
    }

    pub fn with_extension(&self, ext: Substitution) -> Substitution {
        let mut merged = self.0.clone();
        merged.extend(ext.0);
        Substitution(merged)
    }
}

/// A projected result row: selected variable names -> typed bindings.
pub type Row = HashMap<&'static str, Binding>;

pub fn project(sub: &Substitution, select: &[&'static str]) -> Row {
    let mut row = Row::new();
    for name in select {
        if let Some(b) = sub.get(&Var(name)) {
            row.insert(*name, b.clone());
        }
    }
    row
}

/// True when two bindings name the same logical constant (the coercion matrix).
pub fn unify_bindings(a: &Binding, b: &Binding) -> Option<Binding> {
    if a == b {
        return Some(a.clone());
    }
    if let (Some(x), Some(y)) = (a.as_ein(), b.as_ein())
        && x == y
    {
        return Some(Binding::Ein(x));
    }
    if let (Some(x), Some(y)) = (a.as_attr(), b.as_attr())
        && x == y
    {
        return Some(Binding::Attr(x));
    }
    None
}

pub trait Coerce: Clone {
    fn from_binding(b: &Binding) -> Option<Self>;
}

impl Coerce for Ein {
    fn from_binding(b: &Binding) -> Option<Ein> {
        b.as_ein()
    }
}

impl Coerce for AttrEin {
    fn from_binding(b: &Binding) -> Option<AttrEin> {
        b.as_attr()
    }
}

impl Coerce for Val {
    fn from_binding(b: &Binding) -> Option<Val> {
        Some(b.to_val())
    }
}

fn ground_term<T: Coerce>(term: &Term<T>, sub: &Substitution) -> Option<Term<T>> {
    match term {
        Term::Lit(l) => Some(Term::Lit(l.clone())),
        Term::Var(v) => match sub.get(v) {
            None => Some(Term::Var(*v)),
            Some(b) => T::from_binding(b).map(Term::Lit),
        },
    }
}

fn bind_subject(s: &mut Substitution, t: &Term<Ein>, e: Ein) -> Option<()> {
    match t {
        Term::Lit(l) => (l == &e).then_some(()),
        Term::Var(v) => s.bind(*v, Binding::Ein(e)),
    }
}

fn bind_attr(s: &mut Substitution, t: &Term<AttrEin>, a: AttrEin) -> Option<()> {
    match t {
        Term::Lit(l) => (l == &a).then_some(()),
        Term::Var(v) => s.bind(*v, Binding::Attr(a)),
    }
}

fn bind_value(s: &mut Substitution, t: &Term<Val>, v: &Val) -> Option<()> {
    match t {
        Term::Lit(l) => (l == v).then_some(()),
        Term::Var(x) => s.bind(*x, Binding::Val(v.clone())),
    }
}

impl Pattern {
    /// Instantiate known bindings into literals (substitute), failing on a type clash.
    pub fn ground(&self, sub: &Substitution) -> Option<Pattern> {
        Some(Pattern {
            e: ground_term(&self.e, sub)?,
            a: ground_term(&self.a, sub)?,
            v: ground_term(&self.v, sub)?,
        })
    }

    /// Unify this pattern against a ground fact, returning the fresh bindings.
    pub fn unify(&self, fact: &crate::Fact) -> Option<Substitution> {
        let mut s = Substitution::new();
        bind_subject(&mut s, &self.e, fact.e)?;
        bind_attr(&mut s, &self.a, fact.a)?;
        bind_value(&mut s, &self.v, &fact.v)?;
        Some(s)
    }
}
