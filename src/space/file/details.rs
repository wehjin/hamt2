use crate::space::TableAddr;
use crate::FileError;
use redb::{Database, ReadableDatabase, TableDefinition, WriteTransaction};
use serde::{Deserialize, Serialize};

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

    const DETAILS_TABLE: TableDefinition<'static, &str, Vec<u8>> = TableDefinition::new("details");
    pub fn write(self: &Self, write: &WriteTransaction) -> Result<(), FileError> {
        let mut table = write.open_table(Self::DETAILS_TABLE)?;
        let bytes = postcard::to_allocvec(self)?;
        table.insert("main", bytes)?;
        Ok(())
    }
    pub fn read(db: &Database) -> Result<Self, FileError> {
        let read = db.begin_read()?;
        let table = read.open_table(Self::DETAILS_TABLE)?;
        let bytes = table.get("main")?.expect("get details").value();
        let details: Self = postcard::from_bytes(&bytes)?;
        Ok(details)
    }
}
