use crate::hamt::space::addr::Addr;
use crate::hamt::space::mem::MemSegment;
use crate::hamt::space::{Read, ReadError};
use crate::hamt::value::Value;
use std::rc::Rc;
use crate::hamt::space::core::TableItem;

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
    
    fn read_item(&self, addr: Addr) -> Result<&TableItem, ReadError> {
        let Addr::Table(seg, pos) = addr else {
            return Err(ReadError::InvalidAddr(addr));
        };
        let segment = &self.segments[seg.0 as usize];
        let item = segment.read_item(pos)?;
        Ok(item)
    }
}

impl Reader {
    pub fn new(segments: Vec<Rc<MemSegment>>) -> Self {
        Self { segments }
    }
}
