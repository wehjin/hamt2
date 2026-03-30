use crate::space::block::store::Details;
use crate::space::block::store::{Block, BlockStore};
use crate::space::file::block_table::BlockTable;
use crate::space::file::details_table::DetailsTable;
use crate::space::TableAddr;
use redb::{Database, ReadableDatabase};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct RedBlockStore {
    db: Rc<Database>,
}

impl RedBlockStore {
    pub fn new(db: Database) -> Self {
        Self { db: Rc::new(db) }
    }
}

impl BlockStore for RedBlockStore {
    fn write_details(&self, details: &Details) {
        let write = self.db.begin_write().expect("begin write");
        DetailsTable::write(details, &write).expect("write details");
        write.commit().expect("commit");
    }

    fn write_block_details(&self, block: Block, details: &Details) {
        let write = self.db.begin_write().expect("begin write");
        {
            BlockTable::write_slots(&write, block.start_addr, block.slots);
            DetailsTable::write(details, &write).expect("write details");
        }
        write.commit().expect("commit");
    }

    fn read_block(&self, addr: TableAddr) -> Option<Block> {
        let read = self.db.begin_read().expect("begin read");
        let entry = BlockTable::read_slots(&read, addr);
        let block = entry.map(|(start_index, slots)| Block {
            start_addr: TableAddr(start_index),
            slots,
        });
        block
    }

    fn read_details(&self) -> Details {
        let read = self.db.begin_read().expect("begin read");
        let details = DetailsTable::read(&read).expect("read details");
        details
    }
}
