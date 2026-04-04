use crate::db::Db;
use crate::space::Space;
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

    pub async fn solve<T: Space>(self, db: &Db<T>) -> KnowledgeBase<'_, T> {
        for rule in &self.rules {
            if !rule.is_range_restricted() {
                panic!("The program is not range restricted: {:?}", rule);
            }
        }
        let mut kb = KnowledgeBase::from_facts(db, self.facts);
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
    use crate::db::find::program::term::Term;
    use crate::db::{Attr, Datom, Db, Ent, Val};
    use crate::space::mem::MemSpace;

    const ADVISOR: Attr = Attr("member", "advisor");
    const NAME: Attr = Attr("member", "name");
    const QUERY: Attr = Attr("query", "1");

    #[tokio::test]
    async fn program_test() -> anyhow::Result<()> {
        let schema = vec![ADVISOR, NAME];
        let space: MemSpace;
        {
            let mut db = Db::new(MemSpace::new(), schema.clone()).await?;
            db = db
                .transact([
                    Datom::Add(Ent::from(100), ADVISOR, Val::from(103)),
                    Datom::Add(Ent::from(101), ADVISOR, Val::from(103)),
                ])
                .await?;
            space = db.close();
        }
        let db = Db::load(space, schema).await?;

        let program = Program::new(
            [],
            [Rule::new(
                Atom::new(QUERY, [Term::var("x")]),
                [Atom::new(ADVISOR, [Term::var("x"), Term::var("y")])],
            )],
        );
        let kb = program.solve(&db).await;
        let query_result = kb.query(QUERY);
        let mut answers = query_result.into_iter().flatten().collect::<Vec<_>>();
        answers.sort();
        dbg!(&answers);
        assert_eq!(vec![Val::from(100), Val::from(101)], answers);
        Ok(())
    }
}
