use crate::db::component::trie;
use crate::db::find::{EntsWithAttr, Rule, ValsWithEntAttr};
use crate::db::{Attr, Txid, Val};
use crate::space::Space;
use crate::trie::SpaceTrie;
use crate::{LoadError, TransactError};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut, Index};
use crate::db::Eid;

#[derive(Debug)]
pub struct Schema {
    map: HashMap<Attr, Eid>,
}

impl Schema {
    fn basic() -> Self {
        Self {
            map: HashMap::from([(Attr::DB_IDENT, Eid::DB_IDENT)]),
        }
    }
    pub fn new(attrs: Vec<Attr>, eids: Vec<Eid>) -> Self {
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
            trie = trie::trie_add(trie, &self.map, *a_ent, Attr::DB_IDENT, ident, &txid).await?;
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
        let ents_with_idents = {
            let mut rule = EntsWithAttr::new("e", Attr::DB_IDENT);
            rule.update(&trie, &schema).await?;
            rule.results("e").to_vec()
        };
        for ent_with_ident in ents_with_idents {
            let mut rule = ValsWithEntAttr::new("v", ent_with_ident, Attr::DB_IDENT);
            rule.update(&trie, &schema).await?;
            let ident = rule
                .results("v")
                .first()
                .expect("val should exist")
                .as_str();
            if let Some(attr) = attrs_by_ident.get(ident) {
                schema.insert(*attr, ent_with_ident.to_eid());
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
    type Output = Eid;
    fn index(&self, key: Attr) -> &Self::Output {
        &self.map[&key]
    }
}

impl Deref for Schema {
    type Target = HashMap<Attr, Eid>;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for Schema {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
