use crate::space::core::reader::SlotValue;
use crate::space::{Read, Space, TableAddr};
use crate::trie::core::map::TrieMap;
use crate::trie::core::map_base::TrieMapBase;
use crate::trie::mem::base::MemBase;
use crate::trie::mem::slot::MemSlot;
use crate::trie::space::key_value::SpaceKeyValue;
use crate::trie::space::slots::SpaceSlot;
use crate::{space, QueryError, TransactError};

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
                    let u32 = value.save(extend)?;
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
        let Some(map_base) = space_slot.try_map_base() else {
            panic!(
                "Slot value should be a map base, instead: {:?}",
                &slot_value
            );
        };
        map_base
    }
}

impl SpaceMapBase {
    pub async fn into_mem(self, reader: &impl Read) -> Result<TrieMapBase, QueryError> {
        let map = self.to_map();
        let base = self.extract_base();
        let mut mem_slots = Vec::new();
        for i in 0..map.slot_count() {
            let slot = base.read_slot(reader, i).await?;
            if let Some(key_value) = slot.try_key_value() {
                let mem_slot = key_value.to_mem_slot();
                mem_slots.push(mem_slot);
            } else if let Some(map_base) = slot.try_map_base() {
                let trie_map_base = Box::pin(map_base.into_mem(reader)).await?;
                let mem_slot = MemSlot::MapBase(trie_map_base);
                mem_slots.push(mem_slot);
            }
        }
        let mem_map_base = TrieMapBase {
            map,
            base: MemBase { slots: mem_slots },
        };
        Ok(mem_map_base)
    }

    pub fn into_map_base_addr(self) -> (TrieMap, TableAddr) {
        let map = self.to_map();
        let base_addr = self.to_base_addr();
        (map, base_addr)
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
