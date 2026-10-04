use crate::attribute::AttributeDetails;
use crate::cardinality::Cardinality;
use crate::{Bind, Pod};
use futures::StreamExt;
use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Eq, PartialEq)]
pub struct Attribute {
    details: AttributeDetails,
    pod: Pod,
}

impl Attribute {
    pub fn new(details: AttributeDetails, pod: Pod) -> Self {
        Self { details, pod }
    }
    pub fn ident(&self) -> &str {
        self.details.ident()
    }
    pub fn cardinality(&self) -> Cardinality {
        self.details.cardinality()
    }
    pub async fn list_binds(&self) -> Vec<Bind> {
        let attr = self.details.attr();
        let stream = self.pod.ev_stream(attr);
        tokio::pin!(stream);
        stream
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(Bind::from)
            .collect::<Vec<_>>()
    }
}

impl Ord for Attribute {
    fn cmp(&self, other: &Self) -> Ordering {
        self.details.cmp(&other.details)
    }
}

impl PartialOrd for Attribute {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.details.partial_cmp(&other.details)
    }
}
impl fmt::Debug for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Attribute")
            .field("details", &self.details)
            .finish()
    }
}
