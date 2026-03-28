use crate::db::attr::Attr;
use crate::pull::errors::RegisterError;
use crate::pull::pull::Pull;
use std::collections::HashMap;

pub struct Register {
    attrs: HashMap<String, Attr>,
}

impl Register {
    pub fn new() -> Self {
        Self {
            attrs: HashMap::new(),
        }
    }

    pub fn register<T: Pull>(mut self) -> Result<Self, RegisterError> {
        for attr in T::attrs() {
            let ident = attr.to_ident();
            if self.attrs.contains_key(&ident) {
                return Err(RegisterError::DuplicateAttr(attr));
            }
            self.attrs.insert(ident.to_string(), attr);
        }
        Ok(self)
    }
    pub fn to_attrs(&self) -> Vec<Attr> {
        self.attrs.values().cloned().collect()
    }
}
