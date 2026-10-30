#![allow(dead_code)] // the rules engine is currently exercised only by program_test

use std::collections::HashSet;

use super::binding::{Head, Pattern, RawPattern, resolve};
use super::kb::KnowledgeBase;

pub fn rule(head: impl Into<Head>, body: impl Into<Vec<RawPattern>>) -> Rule {
    Rule {
        head: head.into(),
        body: body.into(),
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub head: Head,
    pub body: Vec<RawPattern>,
}

impl Rule {
    /// A rule is range-restricted when every head variable is bound in the body.
    /// This is the termination/domain-independence guard from minimal Datalog.
    pub fn is_range_restricted(&self) -> bool {
        let body_vars: HashSet<_> = self.body.iter().flat_map(|p| p.vars()).collect();
        self.head.vars().iter().all(|v| body_vars.contains(v))
    }

    fn resolved_body(&self, kb: &KnowledgeBase) -> Vec<Pattern> {
        self.body
            .iter()
            .filter_map(|p| resolve(p.clone(), kb.pod.schema()))
            .collect()
    }

    pub fn derive_facts(&self, kb: &KnowledgeBase) -> Vec<Head> {
        let body = self.resolved_body(kb);
        let mut facts = Vec::new();
        for sub in kb.derive_body_subs(&body) {
            let head = self.head.ground(&sub);
            if head.is_grounded() {
                facts.push(head);
            }
        }
        facts
    }
}
