use crate::space::extend::Extend;
use crate::space::reader::Reader;
use crate::space::seg::Seg;
use crate::space::value::Val;
use crate::space::value::Value;
use crate::space::ReadError;
use std::cell::RefCell;

use crate::space::table::{TablePos, TableRoot};
use crate::hamt::trie::mem::slot::MemSlot;
use crate::TransactError;
use std::rc::Rc;

#[derive(Debug)]
pub struct MemSpace {
    segments: RefCell<Vec<Rc<MemSegment>>>,
}
impl MemSpace {
    pub fn new() -> Self {
        Self {
            segments: RefCell::new(Vec::new()),
        }
    }
    pub fn max_seg(&self) -> Seg {
        let count = self.segments.borrow().len();
        let seq = Seg(count as u32);
        seq
    }

    pub fn read(&self) -> Result<Reader, ReadError> {
        let segments = self.segments.borrow().clone();
        let reader = Reader::new(segments);
        Ok(reader)
    }

    pub fn extend(&self) -> Extend {
        Extend::new(self.max_seg())
    }

    pub fn add_segment(
        &mut self,
        seg: Seg,
        values: Vec<Value>,
        table: Vec<MemSlot>,
        root: Option<TableRoot>,
    ) -> Result<(), TransactError> {
        let max_seg = self.max_seg();
        if seg != max_seg {
            return Err(TransactError::SegConflict(seg));
        }
        let segment = MemSegment {
            values,
            table,
            root,
        };
        self.segments.borrow_mut().push(Rc::new(segment));
        Ok(())
    }
}

#[derive(Debug, Default)]
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
