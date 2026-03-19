use crate::core::value::Value;
use crate::hamt::space::core::TableItem;
use crate::hamt::space::mem::MemSegment;
use crate::hamt::space::{Read, ReadError, TableAddr, ValueAddr};
use std::rc::Rc;

pub struct Reader {
    segments: Vec<Rc<MemSegment>>,
}

impl Read for Reader {
    fn read_value(&self, addr: ValueAddr) -> Result<Value, ReadError> {
        let ValueAddr(seg, val) = addr;
        let segment = &self.segments[seg.0 as usize];
        let value = segment.read_value(val)?;
        Ok(value)
    }

    fn read_item(&self, addr: TableAddr) -> Result<&TableItem, ReadError> {
        let TableAddr(seg, pos) = addr;
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
