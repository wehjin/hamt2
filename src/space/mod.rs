mod extend;
pub use extend::Extend;

mod addr;
use crate::error::ReadError;
pub use addr::*;

pub mod file;
pub mod mem;
pub mod reader;
pub mod seg;
pub mod table;
pub mod value;

use crate::hamt::trie::mem::slot::MemSlot;
use crate::space::seg::Seg;
use crate::TransactError;
pub use reader::Reader;
use table::TableRoot;
pub use value::Value;

pub trait Space {
    fn max_seg(&self) -> Seg;
    fn extend(&self) -> Extend {
        Extend::new(self.max_seg())
    }
    fn add_segment(
        &mut self,
        seg: Seg,
        values: Vec<Value>,
        table: Vec<MemSlot>,
        root: Option<TableRoot>,
    ) -> Result<(), TransactError>;
    fn read(&self) -> Result<Reader, ReadError>;
}

pub trait Read {
    fn read_value(&self, addr: ValueAddr) -> Result<Value, ReadError>;
    fn read_slot(&self, addr: &TableAddr, offset: usize) -> Result<&MemSlot, ReadError>;
    fn read_root(&self) -> Result<&Option<TableRoot>, ReadError>;
}

#[cfg(test)]
mod tests {
    use crate::space::file::FileSpace;
    use crate::space::mem::MemSpace;
    use crate::space::seg::Seg;
    use crate::space::value::Value;
    use crate::space::Space;
    use crate::space::{Read, ValueAddr};

    #[tokio::test]
    async fn mem_space_works() {
        let mut space = MemSpace::new();
        let addr = {
            let mut extend = space.extend();
            let value_addr = extend.add_value(Value::U32(42));
            assert_eq!(Value::U32(42), extend.read_value(value_addr).unwrap());
            extend.commit(&mut space).unwrap();
            value_addr
        };
        let reader = space.read().unwrap();
        assert_eq!(Value::U32(42), reader.read_value(addr).unwrap());
    }

    #[tokio::test]
    async fn file_space_works() {
        let addr: ValueAddr;
        let file = tempfile::NamedTempFile::new().expect("tempfile");
        {
            let mut space = FileSpace::new(file.path()).expect("create red space");
            assert_eq!(Seg(0), space.max_seg());
            addr = {
                let mut extend = space.extend();
                let value_addr = extend.add_value(Value::U32(42));
                extend.commit(&mut space).unwrap();
                value_addr
            };
            let reader = space.read().expect("read red space");
            assert_eq!(Value::U32(42), reader.read_value(addr).unwrap());
        }
        {
            let space = FileSpace::load(file.path()).expect("load red space");
            let reader = space.read().expect("read red space");
            assert_eq!(Value::U32(42), reader.read_value(addr).unwrap());
        }
    }
}
