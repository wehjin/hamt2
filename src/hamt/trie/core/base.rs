use crate::client::{QueryError, TransactError};
use crate::core::value::Value;
use crate::hamt::space;
use crate::hamt::space::TableAddr;
use crate::hamt::trie::core::key::TrieKey;
use crate::hamt::trie::core::map_base::TrieMapBase;
use crate::hamt::trie::core::value::TrieValue;
use crate::hamt::trie::mem::base::MemBase;
use crate::hamt::trie::mem::slot::MemSlot;
use crate::hamt::trie::mem::value::MemValue;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TrieBase {
    Mem(MemBase),
    Space(TableAddr),
}

impl TrieBase {
    pub fn save(self, extend: &mut space::Extend) -> Result<TableAddr, TransactError> {
        match self {
            TrieBase::Space(addr) => Ok(addr),
            TrieBase::Mem(base) => {
                let mut items = vec![];
                for slot in base.slots {
                    match slot {
                        MemSlot::KeyValue(key, value) => {
                            let value_addr = match value {
                                TrieValue::Space(value_addr) => value_addr,
                                TrieValue::Mem(value) => {
                                    let value = match value {
                                        MemValue::U32(value) => Value::U32(value),
                                        MemValue::MapBase(TrieMapBase(map, base)) => {
                                            let base_addr = base.save(extend)?;
                                            Value::MapBase(map.0, base_addr)
                                        }
                                    };
                                    let value_addr = extend.add_value(value);
                                    value_addr
                                }
                            };
                            let item = MemSlot::KeyValue(key, TrieValue::Space(value_addr));
                            items.push(item);
                        }
                        MemSlot::MapBase(TrieMapBase(map, base)) => {
                            let base_addr = base.save(extend)?;
                            let item =
                                MemSlot::MapBase(TrieMapBase(map, TrieBase::Space(base_addr)));
                            items.push(item);
                        }
                    }
                }
                let base_addr = extend.add_items(items);
                Ok(base_addr)
            }
        }
    }
}

impl TrieBase {
    pub fn len(&self) -> usize {
        match self {
            Self::Mem(mem) => mem.len(),
            TrieBase::Space(_) => {
                unimplemented!();
            }
        }
    }

    pub fn as_slot<'a>(
        &'a self,
        base_index: usize,
        reader: &'a impl space::Read,
    ) -> Result<&'a MemSlot, QueryError> {
        match self {
            TrieBase::Mem(base) => base.as_slot(base_index),
            TrieBase::Space(base_addr) => {
                let item = reader.read_slot(base_addr, base_index)?;
                Ok(item)
            }
        }
    }

    pub fn new() -> Self {
        TrieBase::Mem(MemBase::new())
    }

    pub fn new_kv(key: TrieKey, value: TrieValue) -> Result<Self, TransactError> {
        Ok(TrieBase::Mem(MemBase::new_kv(key, value)?))
    }

    pub fn new_slot(slot: MemSlot) -> Result<Self, TransactError> {
        let mem_base = MemBase { slots: vec![slot] };
        Ok(TrieBase::Mem(mem_base))
    }

    pub fn insert_kv(
        self,
        base_index: usize,
        key: TrieKey,
        value: TrieValue,
    ) -> Result<Self, TransactError> {
        match self {
            TrieBase::Space(_) => unimplemented!(),
            TrieBase::Mem(mem) => {
                let base = MemBase::insert_kv(mem, base_index, key, value)?;
                Ok(TrieBase::Mem(base))
            }
        }
    }

    pub fn replace_value(self, base_index: usize, value: TrieValue) -> Result<Self, TransactError> {
        match self {
            TrieBase::Space(_) => unimplemented!(),
            TrieBase::Mem(base) => {
                let base = MemBase::replace_value(base, base_index, value)?;
                Ok(TrieBase::Mem(base))
            }
        }
    }

    pub fn merge_kv(
        self,
        base_index: usize,
        key: TrieKey,
        value: TrieValue,
        reader: &impl space::Read,
    ) -> Result<Self, TransactError> {
        match self {
            TrieBase::Space(_) => unimplemented!(),
            TrieBase::Mem(base) => {
                let base = MemBase::merge_kv(base, base_index, key, value, reader)?;
                Ok(TrieBase::Mem(base))
            }
        }
    }
}
