mod extend;

pub use extend::Extend;
use std::fmt::Debug;

mod addr;
use crate::error::ReadError;
pub use addr::*;

pub mod file;
pub mod mem;
pub mod reader;
pub mod seg;
pub mod table;
pub mod value;

use crate::space::reader::SlotValue;
use crate::TransactError;
pub use reader::MemReader;
pub use value::Value;

pub trait Space: Debug + Sized {
    type Reader: Read + Clone + Debug;

    fn add_segment(
        &mut self,
        start_addr: TableAddr,
        slots: Vec<SlotValue>,
        root: Option<TableAddr>,
    ) -> Result<(), TransactError>;
    fn read(&self) -> Result<Self::Reader, ReadError>;
    fn max_addr(&self) -> TableAddr;
    fn extend(&self) -> Result<Extend<Self>, TransactError> {
        let reader = self.read()?;
        Ok(Extend::new(self.max_addr(), reader))
    }
}

pub trait Read {
    fn read_slot(&self, addr: &TableAddr, offset: usize) -> Result<SlotValue, ReadError>;
    fn read_root(&self) -> Result<&Option<TableAddr>, ReadError>;
}

#[cfg(test)]
mod tests {
    use crate::space::file::FileSpace;
    use crate::space::mem::MemSpace;
    use crate::space::reader::SlotValue;
    use crate::space::{Read, Space, TableAddr};

    #[tokio::test]
    async fn mem_space_works() {
        let addr: TableAddr;
        {
            let mut space = MemSpace::new();
            assert_eq!(TableAddr::ZERO, space.max_addr());
            {
                let mut extend = space.extend().unwrap();
                let slot = SlotValue::from((1, 2));
                addr = extend.add_slots(vec![slot]);
                extend.commit(&mut space).unwrap();
            }
            let reader = space.read().unwrap();
            let slot = reader.read_slot(&addr, 0).unwrap();
            assert_eq!(SlotValue::from((1, 2)), slot);
        }
    }

    #[tokio::test]
    async fn file_space_works() {
        let file = tempfile::NamedTempFile::new().expect("tempfile");
        let addr: TableAddr;
        {
            let mut space = FileSpace::new(file.path()).expect("create red space");
            assert_eq!(TableAddr::ZERO, space.max_addr());
            {
                let mut extend = space.extend().unwrap();
                let slot = SlotValue::from((1, 2));
                addr = extend.add_slots(vec![slot]);
                extend.commit(&mut space).unwrap();
            }
            let reader = space.read().unwrap();
            let slot = reader.read_slot(&addr, 0).unwrap();
            assert_eq!(SlotValue::from((1, 2)), slot);
        }
        {
            let space = FileSpace::load(file.path()).expect("load red space");
            let reader = space.read().expect("read red space");
            let slot = reader.read_slot(&addr, 0).unwrap();
            assert_eq!(SlotValue::from((1, 2)), slot);
        }
    }
}
