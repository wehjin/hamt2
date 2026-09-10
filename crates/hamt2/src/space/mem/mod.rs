use crate::error::ReadError;
use crate::space::core::reader::{MemReader, SlotTable, SlotValue};
use serde::{Deserialize, Serialize};

use crate::TransactError;
use crate::space::{Space, TableAddr};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemSpace {
    slots: SlotTable,
    root: Option<TableAddr>,
}
impl MemSpace {
    pub fn new() -> Self {
        Self {
            slots: SlotTable::new(),
            root: None,
        }
    }
}
impl Space for MemSpace {
    type Reader = MemReader;
    async fn add_segment(
        &mut self,
        start_addr: TableAddr,
        slots: Vec<SlotValue>,
        root: Option<TableAddr>,
    ) -> Result<(), TransactError> {
        if start_addr != self.max_addr() {
            return Err(TransactError::InvalidStartAddr(start_addr));
        }
        self.slots.extend(slots);
        self.root = root;
        Ok(())
    }

    async fn read(&self) -> Result<Self::Reader, ReadError> {
        let reader = MemReader::new(self.slots.clone(), self.root);
        Ok(reader)
    }

    fn max_addr(&self) -> TableAddr {
        let len = self.slots.len();
        TableAddr(len as u32)
    }
}
