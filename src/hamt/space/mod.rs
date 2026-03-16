use crate::hamt::value::Value;
use addr::Addr;
use thiserror::Error;

mod extend;
pub use extend::Extend;

pub mod mem;
mod reader;
use crate::hamt::space::seg::Seg;
use crate::hamt::space::val::Val;
pub use reader::Reader;

pub mod addr;
pub mod seg;
pub mod val;

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
    fn read_value(&self, addr: Addr) -> Result<Value, ReadError>;
}

#[cfg(test)]
mod tests {
    use crate::hamt::space::mem::MemSpace;
    use crate::hamt::space::Read;
    use crate::hamt::value::Value;

    #[tokio::test]
    async fn it_works() {
        let mut space = MemSpace::new();
        let addr = {
            let mut extension = space.extend().unwrap();
            let addr = extension.add_value(Value::Uint(42));
            assert_eq!(Value::Uint(42), extension.read_value(addr).unwrap());
            extension.commit(&mut space).unwrap();
            addr
        };
        let reader = space.read().unwrap();
        assert_eq!(Value::Uint(42), reader.read_value(addr).unwrap());
    }
}
