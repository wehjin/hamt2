use crate::space::{Read, TableAddr};
use crate::trie::core::query::QueryKeysValues;
use crate::trie::mem::value::MemValue;
use crate::trie::space::map_base::SpaceMapBase;
use crate::trie::space::slots::SpaceSlot;
use crate::QueryError;

struct Job {
    slot_count: usize,
    slot_offset: usize,
    base_addr: TableAddr,
}
impl Job {
    pub fn is_done(&self) -> bool {
        self.slot_offset >= self.slot_count
    }
    pub fn into_next(mut self) -> Option<Self> {
        self.slot_offset += 1;
        match self.is_done() {
            true => None,
            false => Some(self),
        }
    }
}

impl QueryKeysValues for SpaceMapBase {
    async fn query_keys_values(
        &self,
        reader: &impl Read,
    ) -> Result<Vec<(i32, MemValue)>, QueryError> {
        let mut queue: Vec<Job> = {
            let job = Job {
                slot_count: self.map.slot_count(),
                slot_offset: 0,
                base_addr: self.base_addr,
            };
            match job.is_done() {
                true => vec![],
                false => vec![job],
            }
        };
        let mut out = Vec::new();
        while let Some(current) = queue.pop() {
            let new = if current.slot_offset < current.slot_count {
                let slot = reader
                    .read_slot(&current.base_addr, current.slot_offset)
                    .await?;
                let space_slot = SpaceSlot::assert(slot);
                if let Some(key_value) = space_slot.to_key_value() {
                    let key = key_value.to_key();
                    let value = key_value.to_value();
                    out.push((key, MemValue::U32(value)));
                    None
                } else if let Some(map_base) = space_slot.to_map_base() {
                    let slot_count = map_base.to_map().slot_count();
                    if slot_count > 0 {
                        Some(Job {
                            slot_count,
                            slot_offset: 0,
                            base_addr: map_base.to_base_addr(),
                        })
                    } else {
                        None
                    }
                } else {
                    unreachable!()
                }
            } else {
                None
            };
            if let Some(next) = current.into_next() {
                queue.push(next);
            }
            if let Some(new) = new {
                queue.push(new);
            }
        }
        Ok(out)
    }
}
