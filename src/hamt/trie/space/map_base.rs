use crate::hamt::trie::core::key::TrieKey;
use crate::hamt::trie::core::map::TrieMap;
use crate::hamt::trie::core::map_base::TrieMapBase;
use crate::hamt::trie::mem::base::MemBase;
use crate::hamt::trie::mem::slot::MemSlot;
use crate::hamt::trie::mem::value::MemValue;
use crate::space::reader::SlotValue;
use crate::space::{Read, Space, TableAddr};
use crate::{space, QueryError, TransactError};

pub struct SpaceMapBase(SlotValue);

impl SpaceMapBase {
    pub fn new(map: TrieMap, base_addr: TableAddr) -> Self {
        let base_addr = base_addr.0;
        debug_assert_eq!(0, base_addr & 0x8000_0000);
        Self(SlotValue(map.u32(), base_addr | 0x80000000))
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

    pub fn into_slot_value(self) -> SlotValue {
        self.0
    }
    pub fn assert(slot_value: SlotValue) -> Self {
        debug_assert!(SpaceSlot(slot_value).is_map_base());
        Self(slot_value)
    }
    pub fn load(reader: &impl Read, addr: TableAddr) -> Result<Self, QueryError> {
        let slot_value = reader.read_slot(&addr, 0)?;
        Ok(Self::assert(slot_value))
    }
    pub fn query_value(
        &self,
        key: TrieKey,
        reader: &impl Read,
    ) -> Result<Option<MemValue>, QueryError> {
        let slot = self.to_slot(key, reader)?;
        if let Some(slot) = slot {
            if let Some(key_value) = slot.to_key_value() {
                let value = key_value.query_value(key, reader)?;
                return Ok(value);
            } else if let Some(map_base) = slot.to_map_base() {
                let value = map_base.query_value(key.next(), reader)?;
                return Ok(value);
            }
        }
        Ok(None)
    }
    pub fn query_key_values(&self, reader: &impl Read) -> Result<Vec<(i32, MemValue)>, QueryError> {
        let slot_count = self.to_map().slot_count();
        let base = self.extract_base();
        let mut out = Vec::new();
        for i in 0..slot_count {
            let slot = base.read_slot(reader, i)?;
            if let Some(key_value) = slot.to_key_value() {
                out.push(key_value.to_key_and_value(reader)?);
            } else if let Some(map_base) = slot.to_map_base() {
                let more = map_base.query_key_values(reader)?;
                out.extend(more);
            }
        }
        Ok(out)
    }
    pub fn top_into_mem(self, reader: &impl Read) -> Result<TrieMapBase, QueryError> {
        let map = self.to_map();
        let base = self.extract_base();
        let mut mem_slots = Vec::new();
        for i in 0..map.slot_count() {
            let slot = base.read_slot(reader, i)?;
            if let Some(key_value) = slot.to_key_value() {
                let mem_slot = key_value.to_mem_slot();
                mem_slots.push(mem_slot);
            } else if let Some(map_base) = slot.to_map_base() {
                let slot_value = map_base.to_slot_value();
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

    pub fn to_slot_value(&self) -> SlotValue {
        self.0
    }
    pub fn to_map(&self) -> TrieMap {
        TrieMap(self.0.left())
    }
    pub fn to_base_addr(&self) -> TableAddr {
        let right = self.0.right();
        debug_assert_eq!(0x80000000, right & 0x80000000);
        TableAddr(right & 0x7fffffff)
    }

    pub fn extract_base(&self) -> SpaceBase {
        let addr = self.to_base_addr();
        SpaceBase(addr)
    }

    pub fn to_slot(
        &self,
        key: TrieKey,
        reader: &impl Read,
    ) -> Result<Option<SpaceSlot>, QueryError> {
        if let Some(base_index) = self.to_map().to_base_index(key) {
            let base = self.extract_base();
            let slot = base.read_slot(reader, base_index)?;
            Ok(Some(slot))
        } else {
            Ok(None)
        }
    }
}

pub struct SpaceKeyValue(SlotValue);

impl SpaceKeyValue {
    pub fn new(key: i32, value: u32) -> Self {
        debug_assert_eq!(0, value & 0x8000_0000);
        let slot_value = SlotValue(key as u32, value);
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

pub struct SpaceSlot(SlotValue);
impl SpaceSlot {
    pub fn assert(slot_value: SlotValue) -> Self {
        Self(slot_value)
    }

    pub fn is_key_value(&self) -> bool {
        (self.0.right() & 0x80000000) == 0
    }
    pub fn is_map_base(&self) -> bool {
        (self.0.right() & 0x80000000) == 0x80000000
    }
    pub fn to_key_value(&self) -> Option<SpaceKeyValue> {
        if self.is_key_value() {
            Some(SpaceKeyValue(self.0))
        } else {
            None
        }
    }
    pub fn to_map_base(&self) -> Option<SpaceMapBase> {
        if self.is_map_base() {
            Some(SpaceMapBase(self.0))
        } else {
            None
        }
    }
}

pub struct SpaceBase(TableAddr);

impl SpaceBase {
    pub fn read_slot(&self, reader: &impl Read, offset: usize) -> Result<SpaceSlot, QueryError> {
        let slot_value = reader.read_slot(&self.0, offset)?;
        Ok(SpaceSlot(slot_value))
    }
}

impl From<SlotValue> for SpaceBase {
    fn from(slot_value: SlotValue) -> Self {
        let right = slot_value.right();
        debug_assert_eq!(0x80000000, right & 0x80000000);
        Self(TableAddr(right & 0x7fffffff))
    }
}
