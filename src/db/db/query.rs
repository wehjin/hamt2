use crate::db::{Attr, Db, Ent, Txid, Val};
use crate::db::find::{Rule, ValsWithEntAttr};
use crate::db::key::KEY_MAX_TXID;
use crate::QueryError;
use crate::space::Space;
use crate::trie::mem::value::MemValue;

impl<T: Space> Db<T> {
    pub async fn max_tx(&self) -> Result<Txid, QueryError> {
        let Some(MemValue::U32(value)) = self.trie.query_value(KEY_MAX_TXID).await? else {
            panic!("max_tx not found");
        };
        Ok(Txid::from(value))
    }

    pub async fn find(&self, rule: &mut impl Rule) -> Result<bool, QueryError> {
        rule.update(&self.trie, &self.schema).await
    }

    pub async fn find_val(&self, e: Ent, a: Attr) -> Result<Option<Val>, QueryError> {
        let mut rule = ValsWithEntAttr::new("v", e, a);
        self.find(&mut rule).await?;
        let vals = rule.results("v");
        match vals.first() {
            None => Ok(None),
            Some(v) => Ok(Some(v.clone())),
        }
    }
}