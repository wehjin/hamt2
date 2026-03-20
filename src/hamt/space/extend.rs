use crate::core::value::Value;
use crate::hamt::space::core::{TablePos, TableRoot, Val};
use crate::hamt::space::mem::MemSpace;
use crate::hamt::space::seg::Seg;
use crate::hamt::space::{ExtendError, Read, ReadError, TableAddr, ValueAddr};
use crate::hamt::trie::mem::slot::MemSlot;

pub struct Extend {
    seg: Seg,
    values: Vec<Value>,
    table: Vec<MemSlot>,
    root: Option<TableRoot>,
}

impl Extend {
    pub fn new(seg: Seg) -> Self {
        Self {
            seg,
            values: Vec::new(),
            table: Vec::new(),
            root: None,
        }
    }
    pub fn add_value(&mut self, value: Value) -> ValueAddr {
        let val = Val(self.values.len() as u16);
        self.values.push(value);
        ValueAddr(self.seg, val)
    }

    pub fn add_items(&mut self, items: Vec<MemSlot>) -> TableAddr {
        let pos = TablePos(self.table.len() as u32);
        self.table.extend(items);
        TableAddr(self.seg, pos)
    }
    pub fn set_root(&mut self, root: TableRoot) {
        self.root = Some(root);
    }
    pub fn commit(self, space: &mut MemSpace) -> Result<Seg, ExtendError> {
        let Self {
            seg,
            values,
            table,
            root,
        } = self;
        space.add_segment(seg, values, table, root)?;
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
    fn read_slot(&self, addr: &TableAddr, offset: usize) -> Result<&MemSlot, ReadError> {
        let TableAddr(seg, pos) = addr;
        if *seg == self.seg {
            Ok(&self.table[pos.0 as usize + offset])
        } else {
            unimplemented!()
        }
    }
    fn read_root(&self) -> Result<&Option<TableRoot>, ReadError> {
        Ok(&self.root)
    }
}
