use crate::core::value::Value;
use crate::hamt::space::core::TableRoot;
use crate::hamt::space::mem::MemSegment;
use crate::hamt::space::{Read, ReadError, TableAddr, ValueAddr};
use crate::hamt::trie::mem::slot::MemSlot;
use std::rc::Rc;

#[derive(Debug)]
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

    fn read_slot(&self, addr: &TableAddr, offset: usize) -> Result<&MemSlot, ReadError> {
        let TableAddr(seg, pos) = addr;
        let segment = &self.segments[seg.0 as usize];
        let item = segment.read_slot(pos, offset);
        Ok(item)
    }

    fn read_root(&self) -> Result<&Option<TableRoot>, ReadError> {
        let result = if let Some(segment) = self.segments.last() {
            segment.read_root()?
        } else {
            &None
        };
        Ok(result)
    }
}

impl Reader {
    pub fn new(segments: Vec<Rc<MemSegment>>) -> Self {
        Self { segments }
    }
}
