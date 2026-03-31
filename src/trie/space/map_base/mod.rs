use crate::space::core::reader::SlotValue;
use crate::space::{Read, Space, TableAddr};
use crate::trie::core::key::TrieKey;
use crate::trie::core::map::TrieMap;
use crate::trie::core::map_base::TrieMapBase;
use crate::trie::mem::base::MemBase;
use crate::trie::mem::slot::MemSlot;
use crate::trie::mem::value::MemValue;
use crate::trie::space::slots::SpaceSlot;
use crate::{space, QueryError, TransactError};

pub mod query;

pub struct SpaceMapBase {
    map: TrieMap,
    base_addr: TableAddr,
}

impl SpaceMapBase {
    pub fn save(
        extend: &mut space::Extend<impl Space>,
        map: TrieMap,
        base: MemBase,
    ) -> Result<Self, TransactError> {
        let mut slot_values: Vec<SlotValue> = vec![];
        for slot in base.slots {
            match slot {
                MemSlot::KeyValue(key, value) => {
                    let u32 = value.into_u32(extend)?;
                    let slot_value = SpaceKeyValue::new(key, u32).into_slot_value();
                    slot_values.push(slot_value);
                }
                MemSlot::MapBase(map_base) => {
                    let space_map_base = map_base.into_space_map_base(extend)?;
                    let slot_value = space_map_base.into_slot_value();
                    slot_values.push(slot_value);
                }
            }
        }
        let space_map_base = SpaceMapBase::new_from_slots(slot_values, map, extend)?;
        Ok(space_map_base)
    }
    pub async fn load(reader: &impl Read, addr: TableAddr) -> Result<Self, QueryError> {
        let slot_value = reader.read_slot(&addr, 0).await?;
        Ok(Self::assert(slot_value))
    }
}

impl SpaceMapBase {
    pub fn new(map: TrieMap, base_addr: TableAddr) -> Self {
        Self { map, base_addr }
    }

    pub fn new_from_slots<T: Space>(
        slot_values: Vec<SlotValue>,
        map: TrieMap,
        extend: &mut space::Extend<T>,
    ) -> Result<Self, TransactError> {
        let base_addr = extend.add_slots(slot_values);
        let map_base = Self::new(map, base_addr);
        Ok(map_base)
    }
}

impl SpaceMapBase {
    pub fn into_slot_value(self) -> SlotValue {
        SpaceSlot::from_map_base(self.map, self.base_addr).into_slot_value()
    }
    pub fn assert(slot_value: SlotValue) -> Self {
        let space_slot = SpaceSlot::assert(slot_value);
        let Some(map_base) = space_slot.to_map_base() else {
            panic!(
                "Slot value should be a map base, instead: {:?}",
                &slot_value
            );
        };
        map_base
    }
}

impl SpaceMapBase {
    pub async fn query_value(
        &self,
        key: TrieKey,
        reader: &impl Read,
    ) -> Result<Option<MemValue>, QueryError> {
        let slot = self.to_slot(key, reader).await?;
        if let Some(slot) = slot {
            if let Some(key_value) = slot.to_key_value() {
                let value = key_value.query_value(key, reader)?;
                return Ok(value);
            } else if let Some(map_base) = slot.to_map_base() {
                let value = Box::pin(map_base.query_value(key.next(), reader)).await?;
                return Ok(value);
            }
        }
        Ok(None)
    }
    pub async fn top_into_mem(self, reader: &impl Read) -> Result<TrieMapBase, QueryError> {
        let map = self.to_map();
        let base = self.extract_base();
        let mut mem_slots = Vec::new();
        for i in 0..map.slot_count() {
            let slot = base.read_slot(reader, i).await?;
            if let Some(key_value) = slot.to_key_value() {
                let mem_slot = key_value.to_mem_slot();
                mem_slots.push(mem_slot);
            } else if let Some(map_base) = slot.to_map_base() {
                let slot_value = map_base.into_slot_value();
                let map_base = TrieMapBase::Space(slot_value);
                let mem_slot = MemSlot::MapBase(map_base);
                mem_slots.push(mem_slot);
            }
        }
        let mem_map_base = TrieMapBase::Mem(map, MemBase { slots: mem_slots });
        Ok(mem_map_base)
    }

    pub fn into_map_base_addr(self) -> (TrieMap, TableAddr) {
        let map = self.to_map();
        let base_addr = self.to_base_addr();
        (map, base_addr)
    }

    pub fn into_trie_map_base(self) -> TrieMapBase {
        let slot_value = self.into_slot_value();
        TrieMapBase::Space(slot_value)
    }

    pub fn as_map(&self) -> &TrieMap {
        &self.map
    }
    pub fn to_map(&self) -> TrieMap {
        self.map
    }
    pub fn to_base_addr(&self) -> TableAddr {
        self.base_addr
    }

    pub fn extract_base(&self) -> SpaceBase {
        let addr = self.to_base_addr();
        SpaceBase(addr)
    }

    pub async fn to_slot(
        &self,
        key: TrieKey,
        reader: &impl Read,
    ) -> Result<Option<SpaceSlot>, QueryError> {
        if let Some(base_index) = self.to_map().try_base_index(key) {
            let base = self.extract_base();
            let slot = base.read_slot(reader, base_index).await?;
            Ok(Some(slot))
        } else {
            Ok(None)
        }
    }
}

pub struct SpaceKeyValue(SlotValue);

impl SpaceKeyValue {
    pub fn new(key: i32, value: u32) -> Self {
        let slot_value = SpaceSlot::from_key_value(key, value).into_slot_value();
        Self(slot_value)
    }
    pub fn into_slot_value(self) -> SlotValue {
        self.0
    }

    pub fn to_key(&self) -> i32 {
        self.0.left() as i32
    }
    pub fn to_value(&self) -> u32 {
        self.0.right() & 0x7fffffff
    }
    pub fn to_key_and_value(&self, _reader: &impl Read) -> Result<(i32, MemValue), QueryError> {
        let key = self.to_key();
        let value = self.to_value();
        Ok((key, MemValue::U32(value)))
    }
    pub fn to_mem_slot(self) -> MemSlot {
        let key = self.to_key();
        let value = self.to_value();
        let slot = MemSlot::KeyValue(key, MemValue::U32(value));
        slot
    }
    pub fn query_value(
        &self,
        key: TrieKey,
        _reader: &impl Read,
    ) -> Result<Option<MemValue>, QueryError> {
        if key.i32() == self.to_key() {
            let value = self.to_value();
            Ok(Some(MemValue::U32(value)))
        } else {
            Ok(None)
        }
    }
}

pub struct SpaceBase(TableAddr);

impl SpaceBase {
    pub async fn read_slot(
        &self,
        reader: &impl Read,
        offset: usize,
    ) -> Result<SpaceSlot, QueryError> {
        let slot_value = reader.read_slot(&self.0, offset).await?;
        Ok(SpaceSlot::assert(slot_value))
    }
}

impl From<SlotValue> for SpaceBase {
    fn from(slot_value: SlotValue) -> Self {
        let right = slot_value.right();
        debug_assert_eq!(0x80000000, right & 0x80000000);
        Self(TableAddr(right & 0x7fffffff))
    }
}
