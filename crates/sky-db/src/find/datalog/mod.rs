use crate::Pod;
use kb::KnowledgeBase;
use rule::Rule;

pub mod binding;
pub mod kb;
pub mod rule;
pub mod term;
pub mod var;

/// A Datalog program: a collection of rules over the EAV trie.
#[allow(dead_code)] // exercised only by program_test (recursion)
pub struct Program {
    rules: Vec<Rule>,
}

#[allow(dead_code)]
impl Program {
    pub fn new(rules: impl Into<Vec<Rule>>) -> Self {
        Self {
            rules: rules.into(),
        }
    }

    pub fn solve(self, pod: Pod) -> KnowledgeBase {
        for rule in &self.rules {
            if !rule.is_range_restricted() {
                panic!("The program is not range restricted: {:?}", rule);
            }
        }
        let mut kb = KnowledgeBase::new(pod);
        loop {
            let new_kb = kb.step(&self.rules);
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
    use crate::Transact;
    use crate::create;
    use crate::datom;
    use crate::find::datalog::binding::{head, pattern};
    use crate::find::datalog::rule::rule;
    use crate::find::datalog::term::term;
    use crate::find::datalog::var::var;
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
        let dir = tempfile::tempdir()?;
        let mut peer = create(dir.path(), schema.clone()).await?;
        peer.transact([
            datom::add("a", name(), val("Alice")),
            datom::add("b", name(), val("Bob")),
            datom::add("c", name(), val("Clark")),
            datom::add("a", advisor(), ent("c")),
            datom::add("b", advisor(), ent("c")),
        ])
        .await?;
        let pod = peer.snapshot().await;
        let query1 = rule(
            head(query_1(), [term(var("name"))]),
            [
                pattern(var("advisor"), advisor(), var("advisee")),
                pattern(var("advisor"), name(), var("name")),
            ],
        );
        let query2 = rule(
            head(query_2(), []),
            [
                pattern(var("a"), name(), val("Alice")),
                pattern(var("c"), name(), val("Clark")),
                pattern(var("a"), advisor(), var("c")),
            ],
        );
        let query3 = rule(
            head(query_3(), []),
            [
                pattern(var("a"), name(), val("Alice")),
                pattern(var("b"), name(), val("Bob")),
                pattern(var("a"), advisor(), var("b")),
            ],
        );
        let program = Program::new([query1, query2, query3]);
        let kb = program.solve(pod.clone());
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
