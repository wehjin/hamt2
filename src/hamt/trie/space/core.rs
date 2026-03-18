use crate::client::{QueryError, TransactError};
use crate::hamt::space;
use crate::hamt::space::core::TableItem;
use crate::hamt::trie::key::TrieKey;
use crate::hamt::trie::map::TrieMap;

pub struct SpaceMapBase(TrieMap, SpaceBase);

impl SpaceMapBase {
    pub fn one_kv(
        key: TrieKey,
        value: u32,
        extend: &mut space::Extend,
    ) -> Result<Self, TransactError> {
        let base = SpaceBase::one_kv(key.i32(), value, extend)?;
        let map = TrieMap::set_key_bit(key);
        Ok(Self(map, base))
    }

    pub fn query_value(
        &self,
        key: TrieKey,
        reader: &impl space::Read,
    ) -> Result<Option<u32>, QueryError> {
        let base_index = self.0.to_base_index(key);
        if let Some(base_index) = base_index {
            let value = self.1.query_value(base_index, reader)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpaceBase(space::Addr);

impl SpaceBase {
    pub fn one_kv(key: i32, value: u32, extend: &mut space::Extend) -> Result<Self, TransactError> {
        if value & 0x80000000 != 0 {
            return Err(TransactError::HighBitInValue(value));
        }
        let value = value << 1;
        let item = TableItem(key, value);
        let addr = extend.add_items(vec![item]);
        Ok(Self(addr))
    }

    pub fn query_value(
        &self,
        base_index: usize,
        reader: &impl space::Read,
    ) -> Result<u32, QueryError> {
        let item_addr = self.0.offset_table(base_index);
        let item = reader.read_item(item_addr)?;
        let value = item.1;
        if value & 0x1 != 0 {
            Err(QueryError::NotAValue(value))
        } else {
            Ok(value >> 1)
        }
    }
}
