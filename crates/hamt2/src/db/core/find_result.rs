use crate::db::Val;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub struct FindResult(Vec<HashMap<String, Val>>);

impl FindResult {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl Deref for FindResult {
    type Target = Vec<HashMap<String, Val>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FindResult {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for FindResult {
    type Item = HashMap<String, Val>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
