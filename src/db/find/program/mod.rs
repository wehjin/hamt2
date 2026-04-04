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

    pub fn solve(self) -> KnowledgeBase {
        for rule in &self.rules {
            if !rule.is_range_restricted() {
                panic!("The program is not range restricted: {:?}", rule);
            }
        }
        fn step(kb: &KnowledgeBase, rules: &Vec<Rule>) -> KnowledgeBase {
            let mut new_facts = Vec::new();
            for rule in rules {
                let rule_facts = rule.derive_facts(&kb);
                new_facts.extend(rule_facts);
            }
            kb.with_facts(new_facts)
        }
        let (mut kb, mut old_kb) = (
            KnowledgeBase::from_facts(self.facts),
            KnowledgeBase::empty(),
        );
        while kb != old_kb {
            let new_db = step(&kb, &self.rules);
            (kb, old_kb) = (new_db, kb);
        }
        kb
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::find::program::term::Term;
    use crate::db::Attr;

    const ADVISOR: Attr = Attr("member", "advisor");
    const QUERY: Attr = Attr("query", "1");

    #[test]
    fn program_test() {
        let program = Program::new(
            [
                Atom::new(ADVISOR, [Term::str_val("Alice"), Term::str_val("Bob")]),
                Atom::new(ADVISOR, [Term::str_val("Cliff"), Term::str_val("Bob")]),
            ],
            [Rule::new(
                Atom::new(QUERY, [Term::var("x")]),
                [Atom::new(ADVISOR, [Term::var("x"), Term::var("y")])],
            )],
        );
        let kb = program.solve();
        let query_result = kb.query(QUERY);
        let mut answers = query_result
            .iter()
            .flatten()
            .map(|x| x.as_str())
            .collect::<Vec<_>>();
        answers.sort();
        assert_eq!(vec!["Alice", "Cliff"], answers);
    }
}
