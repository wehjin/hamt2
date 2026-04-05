use crate::db::component::db_trie;
use crate::db::find::{AnyAttrAny, Find};
use crate::db::Ein;
use crate::db::{Attr, Txid, Val};
use crate::space::Space;
use crate::trie::SpaceTrie;
use crate::{LoadError, TransactError};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut, Index};

#[derive(Debug)]
pub struct Schema {
    map: HashMap<Attr, Ein>,
}

impl Schema {
    fn basic() -> Self {
        Self {
            map: HashMap::from([(Attr::DB_IDENT, Ein::DB_IDENT)]),
        }
    }
    pub fn new(attrs: Vec<Attr>, eids: Vec<Ein>) -> Self {
        let mut schema = Self::basic();
        let more = attrs.iter().cloned().zip(eids).collect::<Vec<_>>();
        schema.extend(more);
        schema
    }
    pub async fn save<T: Space>(
        &self,
        mut trie: SpaceTrie<T>,
        txid: Txid,
    ) -> Result<SpaceTrie<T>, TransactError> {
        for (at, a_ent) in self.map.iter() {
            let ident = Val::from(at.to_ident().as_str());
            trie = db_trie::add(trie, &self.map, *a_ent, Attr::DB_IDENT, ident, &txid).await?;
        }
        Ok(trie)
    }
    pub async fn load<T: Space>(attrs: Vec<Attr>, trie: &SpaceTrie<T>) -> Result<Self, LoadError> {
        let attrs_by_ident: HashMap<String, Attr> = attrs
            .into_iter()
            .map(|attr| (attr.to_ident(), attr))
            .collect();
        // Read attr eids from the trie.
        let mut schema = Schema::basic();
        let ident_evs = AnyAttrAny::new(Attr::DB_IDENT).apply(trie, &schema).await?;
        for (ein, val) in ident_evs {
            let ident = val.as_str();
            if let Some(attr) = attrs_by_ident.get(ident) {
                schema.insert(*attr, ein);
            }
        }
        // Confirm all requested attrs have eids.
        for attr in attrs_by_ident.values() {
            if !schema.contains_key(attr) {
                return Err(LoadError::UnknownAttr(*attr));
            }
        }
        Ok(schema)
    }
}

impl Index<Attr> for Schema {
    type Output = Ein;
    fn index(&self, key: Attr) -> &Self::Output {
        &self.map[&key]
    }
}

impl Deref for Schema {
    type Target = HashMap<Attr, Ein>;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for Schema {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
