use crate::hamt::space::addr::Addr;
use crate::hamt::space::mem::MemSpace;
use crate::hamt::space::seg::Seg;
use crate::hamt::space::val::Val;
use crate::hamt::space::{ExtendError, Read, ReadError};
use crate::hamt::value::Value;

pub struct Extend {
    seg: Seg,
    values: Vec<Value>,
}

impl Read for Extend {
    fn read_value(&self, addr: Addr) -> Result<Value, ReadError> {
        let Addr::Value(seg, val) = addr else {
            return Err(ReadError::InvalidAddr(addr));
        };
        if seg == self.seg {
            Ok(self.values[val.0 as usize].clone())
        } else {
            unimplemented!()
        }
    }
}

impl Extend {
    pub fn new(seg: Seg) -> Self {
        Self {
            seg,
            values: Vec::new(),
        }
    }
    pub fn commit(self, space: &mut MemSpace) -> Result<Seg, ExtendError> {
        let Self { seg, values } = self;
        space.add_segment(seg, values)?;
        Ok(seg)
    }
    pub fn add_value(&mut self, value: Value) -> Addr {
        let val = Val(self.values.len() as u16);
        self.values.push(value);
        Addr::Value(self.seg, val)
    }
}
