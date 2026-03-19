use crate::core::value::Value;
use crate::hamt::space::core::{TableItem, TablePos, Val};
use crate::hamt::space::mem::MemSpace;
use crate::hamt::space::seg::Seg;
use crate::hamt::space::{ExtendError, Read, ReadError, TableAddr, ValueAddr};

pub struct Extend {
    seg: Seg,
    values: Vec<Value>,
    table: Vec<TableItem>,
}

impl Extend {
    pub fn new(seg: Seg) -> Self {
        Self {
            seg,
            values: Vec::new(),
            table: Vec::new(),
        }
    }
    pub fn add_value(&mut self, value: Value) -> ValueAddr {
        let val = Val(self.values.len() as u16);
        self.values.push(value);
        ValueAddr(self.seg, val)
    }
    pub fn add_items(&mut self, items: Vec<TableItem>) -> TableAddr {
        let pos = TablePos(self.table.len() as u32);
        self.table.extend(items);
        TableAddr(self.seg, pos)
    }
    pub fn commit(self, space: &mut MemSpace) -> Result<Seg, ExtendError> {
        let Self { seg, values, table } = self;
        space.add_segment(seg, values, table)?;
        Ok(seg)
    }
}

impl Read for Extend {
    fn read_value(&self, addr: ValueAddr) -> Result<Value, ReadError> {
        let ValueAddr(seg, val) = addr;
        if seg == self.seg {
            Ok(self.values[val.0 as usize].clone())
        } else {
            unimplemented!()
        }
    }

    fn read_item(&self, addr: TableAddr) -> Result<&TableItem, ReadError> {
        let TableAddr(seg, pos) = addr;
        if seg == self.seg {
            Ok(&self.table[pos.0 as usize])
        } else {
            unimplemented!()
        }
    }
}
