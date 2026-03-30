use redb::{ReadTransaction, TableDefinition, WriteTransaction};
use crate::FileError;
use crate::space::core::block_store::Details;

pub struct DetailsTable;

impl DetailsTable {
    const DETAILS_TABLE: TableDefinition<'static, &str, Vec<u8>> = TableDefinition::new("details");
    pub fn write(details: &Details, write: &WriteTransaction) -> Result<(), FileError> {
        let mut table = write.open_table(Self::DETAILS_TABLE)?;
        let bytes = postcard::to_allocvec(details)?;
        table.insert("main", bytes)?;
        Ok(())
    }
    pub fn read(read: &ReadTransaction) -> Result<Details, FileError> {
        let table = read.open_table(Self::DETAILS_TABLE)?;
        let bytes = table.get("main")?.expect("get details").value();
        let details = postcard::from_bytes::<Details>(&bytes)?;
        Ok(details)
    }
}