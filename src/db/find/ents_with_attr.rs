use crate::db::core::datom::attr::Attr;
use crate::db::core::datom::ent::Ent;
use crate::db::find::Rule;
use crate::db::component::key::KEY_AEVT;
use crate::db::Eid;
use crate::db::schema::Schema;
use crate::space::Space;
use crate::trie::SpaceTrie;
use crate::QueryError;

pub struct EntsWithAttr {
    name: &'static str,
    attr: Attr,
    values: Vec<Ent>,
}

impl EntsWithAttr {
    pub fn new(name: &'static str, attr: Attr) -> Self {
        Self {
            name,
            attr,
            values: Vec::new(),
        }
    }
}

impl Rule for EntsWithAttr {
    type Output = Ent;

    fn results(&self, name: &'static str) -> &[Self::Output] {
        if name == self.name { &self.values } else { &[] }
    }
    async fn update<T: Space>(
        &mut self,
        trie: &SpaceTrie<T>,
        schema: &Schema,
    ) -> Result<bool, QueryError> {
        let aid = schema.get(&self.attr).expect("attr should exist").to_i32();
        let aevt_key = [KEY_AEVT, aid];
        let eids = if let Some(value) = trie.deep_query_value(aevt_key).await? {
            let subtrie = trie.to_subtrie_from_value(value).await?;
            let key_values = subtrie.query_keys_values().await?;
            let eids = key_values
                .into_iter()
                .map(|(eid, _)| Ent::Id(Eid(eid)))
                .collect::<Vec<_>>();
            eids
        } else {
            Vec::new()
        };
        if eids == self.values {
            Ok(false)
        } else {
            self.values = eids;
            Ok(true)
        }
    }
}
