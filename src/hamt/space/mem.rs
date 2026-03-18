use crate::hamt::space::core::{TableItem, TablePos, Val};
use crate::hamt::space::extend::Extend;
use crate::hamt::space::reader::Reader;
use crate::hamt::space::seg::Seg;
use crate::hamt::space::{ExtendError, ReadError};
use crate::hamt::trie::mem::value::MemValue;
use std::rc::Rc;

pub struct MemSpace {
    segments: Vec<Rc<MemSegment>>,
}
impl MemSpace {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn extend(&self) -> Result<Extend, ExtendError> {
        let max_seg = Seg(self.segments.len() as u32);
        Ok(Extend::new(max_seg))
    }

    pub fn add_segment(
        &mut self,
        seg: Seg,
        values: Vec<MemValue>,
        table: Vec<TableItem>,
    ) -> Result<(), ExtendError> {
        if seg != Seg(self.segments.len() as u32) {
            return Err(ExtendError::SegConflict(seg));
        }
        let segment = MemSegment { values, table };
        self.segments.push(Rc::new(segment));
        Ok(())
    }

    pub fn read(&self) -> Result<Reader, ReadError> {
        let reader = Reader::new(self.segments.clone());
        Ok(reader)
    }
}

pub struct MemSegment {
    values: Vec<MemValue>,
    table: Vec<TableItem>,
}

impl MemSegment {
    pub fn read_value(&self, val: Val) -> Result<MemValue, ReadError> {
        let value = self
            .values
            .get(val.0 as usize)
            .ok_or(ReadError::InvalidVal(val))?;
        Ok(value.clone())
    }

    pub fn read_item(&self, pos: TablePos) -> Result<&TableItem, ReadError> {
        Ok(&self.table[pos.0 as usize])
    }
}
