use thiserror::Error;

mod extend;
pub use extend::Extend;

mod addr;
pub use addr::*;
pub mod value;
pub mod mem;
pub mod reader;
pub mod seg;
pub mod table;

use crate::hamt::space::value::Val;
use crate::hamt::trie::mem::slot::MemSlot;
pub use value::Value;
pub use reader::Reader;
use table::{TablePos, TableRoot};

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("Invalid table addr {0}")]
    InvalidTableAddr(TableAddr),

    #[error("TablePos {0} with offset {1} exceeded the segment's length {2}")]
    TablePosWithOffsetExceedsSegmentLen(TablePos, usize, usize),

    #[error("Invalid value addr {0}")]
    InvalidValueAddr(ValueAddr),

    #[error("Invalid val {0}")]
    InvalidVal(Val),
}

pub trait Read {
    fn read_value(&self, addr: ValueAddr) -> Result<Value, ReadError>;
    fn read_slot(&self, addr: &TableAddr, offset: usize) -> Result<&MemSlot, ReadError>;
    fn read_root(&self) -> Result<&Option<TableRoot>, ReadError>;
}

#[cfg(test)]
mod tests {
    use crate::hamt::space::value::Value;
    use crate::hamt::space::mem::MemSpace;
    use crate::hamt::space::Read;

    #[tokio::test]
    async fn space_works() {
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
}
