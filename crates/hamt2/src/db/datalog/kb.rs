use crate::db::component::db_trie;
use crate::db::datalog::atom::Atom;
use crate::db::datalog::rule::Rule;
use crate::db::datalog::sub::Substitution;
use crate::db::datalog::term::Term;
use crate::db::{Attr, Schema, Val};
use crate::trie::TrieQuery;
use crate::trie::base_storage::BaseStorageRead;
use async_stream::stream;
use futures::{StreamExt, pin_mut};
use std::collections::HashSet;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct KnowledgeBase<'a, T, S>
where
    T: TrieQuery<S>,
    S: BaseStorageRead,
{
    db_trie: &'a T,
    schema: &'a Schema,
    facts: HashSet<Atom>,
    phantom: PhantomData<&'a S>,
}

impl<'a, T, S> KnowledgeBase<'a, T, S>
where
    T: TrieQuery<S>,
    S: BaseStorageRead,
{
    pub fn from_facts(db_trie: &'a T, schema: &'a Schema, facts: Vec<Atom>) -> Self {
        debug_assert!(facts.iter().all(|atom| atom.is_grounded()));
        Self {
            db_trie,
            schema,
            facts: facts.into_iter().collect(),
            phantom: PhantomData,
        }
    }
    #[must_use]
    pub fn with_facts(&self, new_facts: Vec<Atom>) -> Self {
        debug_assert!(new_facts.iter().all(|atom| atom.is_grounded()));
        let mut facts = self.facts.clone();
        facts.extend(new_facts);
        Self {
            db_trie: self.db_trie,
            schema: self.schema,
            facts,
            phantom: PhantomData,
        }
    }

    #[must_use]
    pub fn query(&self, query: Attr) -> Vec<Vec<Val>> {
        let mut results = Vec::new();
        for atom in &self.facts {
            if atom.attr == query {
                let mut vals = Vec::new();
                for term in atom.terms.iter() {
                    let val = match term {
                        Term::Val(val) => val.clone(),
                        Term::Var(_var) => panic!("Ungrounded atom in kb: {:?}", atom),
                    };
                    vals.push(val);
                }
                results.push(vals);
            }
        }
        results
    }

    fn facts_stream(&self, earth_atom: &Atom) -> impl futures::Stream<Item = Atom> {
        stream! {
            if earth_atom.terms.len() == 2 {
                let ev_stream = db_trie::ev_stream(self.db_trie, earth_atom.attr, self.schema);
                pin_mut!(ev_stream);
                while let Some((e,v)) = ev_stream.next().await {
                    yield Atom::new(earth_atom.attr, [Term::from(e), Term::from(v)]);
                }
            }
            for fact in self.facts.iter() {
                yield fact.clone();
            }
        }
    }

    pub async fn unify_earth_atom(
        &self,
        earth_atom: &Atom,
        grounding_sub: &Substitution,
    ) -> Vec<Substitution> {
        let mut new_subs = Vec::new();
        let facts_stream = self.facts_stream(earth_atom);
        pin_mut!(facts_stream);
        while let Some(kb_atom) = facts_stream.next().await {
            if let Some(extension) = earth_atom.unify(&kb_atom) {
                let new_sub = grounding_sub.with_extension(extension);
                new_subs.push(new_sub);
            }
        }
        new_subs
    }

    pub async fn step(&self, rules: &Vec<Rule>) -> Self {
        let mut new_facts = Vec::new();
        for rule in rules {
            let rule_facts = rule.derive_facts(self).await;
            debug_assert!(rule_facts.iter().all(|atom| atom.is_grounded()));
            new_facts.extend(rule_facts);
        }
        self.with_facts(new_facts)
    }
}

impl<'a, T, S> PartialEq for KnowledgeBase<'a, T, S>
where
    T: TrieQuery<S>,
    S: BaseStorageRead,
{
    fn eq(&self, other: &Self) -> bool {
        self.facts == other.facts
    }
}

impl<'a, T, S> Eq for KnowledgeBase<'a, T, S>
where
    T: TrieQuery<S>,
    S: BaseStorageRead,
{
}
