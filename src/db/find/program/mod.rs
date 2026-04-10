use crate::db::Schema;
use crate::space::Space;
use crate::trie::SpaceTrie;
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

    pub async fn solve<'a, T: Space>(
        self,
        db_trie: &'a SpaceTrie<T>,
        schema: &'a Schema,
    ) -> KnowledgeBase<'a, T> {
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
    use crate::db::find::program::atom::atom;
    use crate::db::find::program::rule::rule;
    use crate::db::find::program::term::term;
    use crate::db::find::program::var::var;
    use crate::db::{datom, ent, val, Attr, Db};
    use crate::space::mem::MemSpace;

    const ADVISOR: Attr = Attr("member/advisor");
    const NAME: Attr = Attr("member/name");
    const QUERY_1: Attr = Attr("query/1");
    const QUERY_2: Attr = Attr("query/2");
    const QUERY_3: Attr = Attr("query/3");

    #[tokio::test]
    async fn program_test() -> anyhow::Result<()> {
        let schema = vec![ADVISOR, NAME];
        let space: MemSpace;
        {
            let mut db = Db::new(MemSpace::new(), schema.clone()).await?;
            db = db
                .transact([
                    datom::add("a", NAME, val("Alice")),
                    datom::add("b", NAME, val("Bob")),
                    datom::add("c", NAME, val("Clark")),
                    datom::add("a", ADVISOR, ent("c")),
                    datom::add("b", ADVISOR, ent("c")),
                ])
                .await?;
            space = db.close();
        }
        let db = Db::load(space, schema).await?;
        let query1 = rule(
            atom(QUERY_1, [term(var("name"))]),
            [
                atom(ADVISOR, [term(var("advisor")), term(var("advisee"))]),
                atom(NAME, [term(var("advisor")), term(var("name"))]),
            ],
        );
        let query2 = rule(
            atom(QUERY_2, []),
            [
                atom(NAME, [term(var("a")), term(val("Alice"))]),
                atom(NAME, [term(var("c")), term(val("Clark"))]),
                atom(ADVISOR, [term(var("a")), term(var("c"))]),
            ],
        );
        let query3 = rule(
            atom(QUERY_3, []),
            [
                atom(NAME, [term(var("a")), term(val("Alice"))]),
                atom(NAME, [term(var("b")), term(val("Bob"))]),
                atom(ADVISOR, [term(var("a")), term(var("b"))]),
            ],
        );
        let program = Program::new([], [query1, query2, query3]);
        let kb = program.solve(&db.trie, &db.schema).await;
        let q1_result = kb.query(QUERY_1);
        let mut answers = q1_result.into_iter().flatten().collect::<Vec<_>>();
        answers.sort();
        assert_eq!(vec![val("Alice"), val("Bob")], answers);
        let q2_result = kb.query(QUERY_2);
        assert_eq!(1, q2_result.len());
        let q3_result = kb.query(QUERY_3);
        assert_eq!(0, q3_result.len());
        Ok(())
    }
}
