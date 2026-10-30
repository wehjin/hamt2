use crate::Pod;
use crate::find::datalog::binding::{Pattern, RawPattern, Row, project, resolve};
use crate::find::datalog::kb::KnowledgeBase;

pub(crate) fn run_find(
    pod: &Pod,
    select: Vec<&'static str>,
    where_: Vec<RawPattern>,
) -> Vec<Row> {
    let body: Vec<Pattern> = where_
        .into_iter()
        .filter_map(|p| resolve(p, pod.schema()))
        .collect();
    let kb = KnowledgeBase::new(pod.clone());
    kb.derive_body_subs(&body)
        .into_iter()
        .map(|sub| project(&sub, &select))
        .collect()
}
