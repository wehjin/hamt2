use crate::space::mem::MemSpace;
use crate::space::seg::Seg;
use crate::space::table::{TablePos, TableRoot};
use crate::space::value::Val;
use crate::space::value::Value;
use crate::space::{Read, ReadError, TableAddr, ValueAddr};
use crate::hamt::trie::mem::slot::MemSlot;
use crate::TransactError;

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
    pub fn commit(self, space: &mut MemSpace) -> Result<(), TransactError> {
        let Extend {
            seg,
            values,
            table,
            root,
        } = self;
        assert_eq!(space.max_seg(), seg);
        space.add_segment(seg, values, table, root)?;
        Ok(())
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
}

impl Read for Extend {
    fn read_value(&self, addr: ValueAddr) -> Result<Value, ReadError> {
        let ValueAddr(seg, val) = addr;
        debug_assert_eq!(seg, self.seg);
        Ok(self.values[val.0 as usize].clone())
    }
    fn read_slot(&self, addr: &TableAddr, offset: usize) -> Result<&MemSlot, ReadError> {
        let TableAddr(seg, pos) = addr;
        debug_assert_eq!(seg, &self.seg);
        Ok(&self.table[pos.0 as usize + offset])
    }
    fn read_root(&self) -> Result<&Option<TableRoot>, ReadError> {
        Ok(&self.root)
    }
}
