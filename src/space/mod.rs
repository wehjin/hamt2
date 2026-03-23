mod extend;
pub use extend::Extend;

mod addr;
pub use addr::*;
use crate::error::ReadError;

pub mod mem;
pub mod reader;
pub mod seg;
pub mod table;
pub mod value;

use crate::hamt::trie::mem::slot::MemSlot;
pub use reader::Reader;
use table::TableRoot;
pub use value::Value;

pub trait Read {
    fn read_value(&self, addr: ValueAddr) -> Result<Value, ReadError>;
    fn read_slot(&self, addr: &TableAddr, offset: usize) -> Result<&MemSlot, ReadError>;
    fn read_root(&self) -> Result<&Option<TableRoot>, ReadError>;
}

#[cfg(test)]
mod tests {
    use crate::space::mem::MemSpace;
    use crate::space::value::Value;
    use crate::space::Read;

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
