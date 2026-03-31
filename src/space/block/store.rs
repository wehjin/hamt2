use crate::space::core::reader::SlotValue;
use crate::space::TableAddr;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub trait BlockStore {
    fn write_details(&self, details: &Details) -> impl Future<Output = ()>;
    fn write_block_details(&self, block: Block, details: &Details) -> impl Future<Output = ()>;
    fn read_block(&self, addr: TableAddr) -> impl Future<Output = Option<Block>>;
    fn read_details(&self) -> impl Future<Output = Details>;
}

pub struct Block {
    pub start_addr: TableAddr,
    pub slots: Vec<SlotValue>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Details {
    pub slot_count: usize,
    pub root: Option<TableAddr>,
}

impl Details {
    pub fn max_addr(&self) -> TableAddr {
        TableAddr::from(self.slot_count)
    }

    pub fn with_update(&self, more_slots: usize, root: Option<TableAddr>) -> Self {
        Self {
            slot_count: self.slot_count + more_slots,
            root,
        }
    }
}
