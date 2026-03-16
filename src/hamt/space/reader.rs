use crate::hamt::space::addr::Addr;
use crate::hamt::space::mem::MemSegment;
use crate::hamt::space::{Read, ReadError};
use crate::hamt::value::Value;
use std::rc::Rc;

pub struct Reader {
    segments: Vec<Rc<MemSegment>>,
}

impl Read for Reader {
    fn read_value(&self, addr: Addr) -> Result<Value, ReadError> {
        let Addr::Value(seg, val) = addr else {
            return Err(ReadError::InvalidAddr(addr));
        };
        let segment = &self.segments[seg.0 as usize];
        let value = segment.read_value(val)?;
        Ok(value)
    }
}

impl Reader {
    pub fn new(segments: Vec<Rc<MemSegment>>) -> Self {
        Self { segments }
    }
}
