use crate::Schema;
use crate::trie::*;
use atom::Atom;
use kb::KnowledgeBase;
use rule::Rule;

pub mod atom;
pub mod kb;
pub mod rule;
pub mod sub;
pub mod term;
pub mod var;

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

    pub async fn solve<'a, T>(self, db_trie: &'a T, schema: &'a Schema) -> KnowledgeBase<'a, T>
    where
        T: QueryCursor + KvStream + Snap + Query,
    {
        for rule in &self.rules {
            if !rule.is_range_restricted() {
                panic!("The program is not range restricted: {:?}", rule);
            }
        }
        let mut kb = KnowledgeBase::from_facts(db_trie, schema, self.facts);
        loop {
            let new_kb = kb.step(&self.rules).await;
            if new_kb == kb {
                return kb;
            } else {
                kb = new_kb;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::_internal::datalog::atom::atom;
    use crate::_internal::datalog::rule::rule;
    use crate::_internal::datalog::term::term;
    use crate::_internal::datalog::var::var;
    use crate::Db;
    use crate::Transact;
    use crate::datom;
    use crate::trie::SkyTrie;
    use crate::{Attr, ent, val};

    fn advisor() -> Attr {
        Attr::from("member/advisor")
    }
    fn name() -> Attr {
        Attr::from("member/name")
    }
    fn query_1() -> Attr {
        Attr::from("query/1")
    }
    fn query_2() -> Attr {
        Attr::from("query/2")
    }
    fn query_3() -> Attr {
        Attr::from("query/3")
    }

    #[tokio::test]
    async fn program_test() -> anyhow::Result<()> {
        let schema = vec![advisor(), name()];
        let mut storage = SkyTrie::new();
        {
            let mut db = Db::new(storage, schema.clone()).await?;
            db.transact([
                datom::add("a", name(), val("Alice")),
                datom::add("b", name(), val("Bob")),
                datom::add("c", name(), val("Clark")),
                datom::add("a", advisor(), ent("c")),
                datom::add("b", advisor(), ent("c")),
            ])
            .await?;
            storage = db.close();
        }
        let db = Db::load(storage).await;
        let query1 = rule(
            atom(query_1(), [term(var("name"))]),
            [
                atom(advisor(), [term(var("advisor")), term(var("advisee"))]),
                atom(name(), [term(var("advisor")), term(var("name"))]),
            ],
        );
        let query2 = rule(
            atom(query_2(), []),
            [
                atom(name(), [term(var("a")), term(val("Alice"))]),
                atom(name(), [term(var("c")), term(val("Clark"))]),
                atom(advisor(), [term(var("a")), term(var("c"))]),
            ],
        );
        let query3 = rule(
            atom(query_3(), []),
            [
                atom(name(), [term(var("a")), term(val("Alice"))]),
                atom(name(), [term(var("b")), term(val("Bob"))]),
                atom(advisor(), [term(var("a")), term(var("b"))]),
            ],
        );
        let program = Program::new([], [query1, query2, query3]);
        let kb = program.solve(&db.trie, &db.schema).await;
        let q1_result = kb.query(query_1());
        let mut answers = q1_result.into_iter().flatten().collect::<Vec<_>>();
        answers.sort();
        assert_eq!(vec![val("Alice"), val("Bob")], answers);
        let q2_result = kb.query(query_2());
        assert_eq!(1, q2_result.len());
        let q3_result = kb.query(query_3());
        assert_eq!(0, q3_result.len());
        Ok(())
    }
}
