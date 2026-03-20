use crate::core::value::Value;
use crate::hamt::space;
use crate::hamt::trie::core::base::TrieBase;
use crate::hamt::trie::core::map::TrieMap;
use crate::hamt::trie::core::map_base::TrieMapBase;
use crate::hamt::trie::mem::value::MemValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrieValue {
    Mem(MemValue),
    Space(space::ValueAddr),
}

impl TrieValue {
    pub fn u32(v: u32) -> Self {
        assert_eq!(
            0,
            v & 0x80000000,
            "TrieValue u32 must not have the high bit set"
        );
        TrieValue::Mem(MemValue::U32(v))
    }

    pub fn to_mem_value(&self, reader: &impl space::Read) -> Result<MemValue, space::ReadError> {
        let value = match self {
            TrieValue::Mem(value) => value.clone(),
            TrieValue::Space(value) => match reader.read_value(*value)? {
                Value::U32(v) => MemValue::U32(v),
                Value::MapBase(map, base_addr) => {
                    let map = TrieMap(map);
                    let base = TrieBase::Space(base_addr);
                    let map_base = TrieMapBase(map, base);
                    MemValue::MapBase(map_base)
                }
            },
        };
        Ok(value)
    }
}
