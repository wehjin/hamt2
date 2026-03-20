use crate::core::value::Value;
use crate::hamt::space::core::{TablePos, TableRoot, Val};
use crate::hamt::space::extend::Extend;
use crate::hamt::space::reader::Reader;
use crate::hamt::space::seg::Seg;
use crate::hamt::space::{ExtendError, ReadError};

use crate::hamt::trie::mem::slot::MemSlot;
use std::rc::Rc;

#[derive(Debug)]
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
        values: Vec<Value>,
        table: Vec<MemSlot>,
        root: Option<TableRoot>,
    ) -> Result<(), ExtendError> {
        if seg != Seg(self.segments.len() as u32) {
            return Err(ExtendError::SegConflict(seg));
        }
        let segment = MemSegment {
            values,
            table,
            root,
        };
        self.segments.push(Rc::new(segment));
        Ok(())
    }

    pub fn read(&self) -> Result<Reader, ReadError> {
        let reader = Reader::new(self.segments.clone());
        Ok(reader)
    }
}

#[derive(Debug)]
pub struct MemSegment {
    values: Vec<Value>,
    table: Vec<MemSlot>,
    root: Option<TableRoot>,
}

impl MemSegment {
    pub fn read_value(&self, val: Val) -> Result<Value, ReadError> {
        let value = self
            .values
            .get(val.0 as usize)
            .ok_or(ReadError::InvalidVal(val))?;
        Ok(value.clone())
    }

    pub fn read_slot(&self, pos: &TablePos, offset: usize) -> &MemSlot {
        let index = pos.0 as usize + offset;
        assert!(index < self.table.len());
        &self.table[index]
    }

    pub fn read_root(&self) -> Result<&Option<TableRoot>, ReadError> {
        Ok(&self.root)
    }
}
