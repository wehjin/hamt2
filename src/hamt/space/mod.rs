use crate::hamt::trie::mem::value::MemValue;
use thiserror::Error;

mod extend;
pub use extend::Extend;
mod addr;
pub use addr::Addr;

pub mod mem;
pub mod reader;
pub use reader::Reader;

pub mod core;
pub mod seg;

use crate::hamt::space::core::{TableItem, Val};
use crate::hamt::space::seg::Seg;

#[derive(Error, Debug)]
pub enum ExtendError {
    #[error("Segment {0} already exists")]
    SegConflict(Seg),
}

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("Invalid addr {0}")]
    InvalidAddr(Addr),

    #[error("Invalid val {0}")]
    InvalidVal(Val),
}

pub trait Read {
    fn read_value(&self, addr: Addr) -> Result<MemValue, ReadError>;
    fn read_item(&self, addr: Addr) -> Result<&TableItem, ReadError>;
}

#[cfg(test)]
mod tests {
    use crate::hamt::space::mem::MemSpace;
    use crate::hamt::space::Read;
    use crate::hamt::trie::mem::value::MemValue;

    #[tokio::test]
    async fn space_works() {
        let mut space = MemSpace::new();
        let addr = {
            let mut extend = space.extend().unwrap();

            let value_addr = extend.add_value(MemValue::U32(42));
            assert_eq!(MemValue::U32(42), extend.read_value(value_addr).unwrap());

            extend.commit(&mut space).unwrap();
            value_addr
        };
        let reader = space.read().unwrap();
        assert_eq!(MemValue::U32(42), reader.read_value(addr).unwrap());
    }
}
