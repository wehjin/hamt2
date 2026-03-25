use crate::space::reader::{SlotTable, SlotValue};
use crate::space::Space;
use crate::space::TableAddr;
use crate::TransactError;

pub struct Extend {
    start_addr: TableAddr,
    slots: SlotTable,
    root: Option<TableAddr>,
}

impl Extend {
    pub fn new(start_addr: TableAddr) -> Self {
        Self {
            start_addr,
            slots: SlotTable::new(),
            root: None,
        }
    }

    fn max_addr(&self) -> TableAddr {
        self.start_addr + self.slots.len()
    }

    pub fn add_slots(&mut self, items: Vec<SlotValue>) -> TableAddr {
        let pos = self.max_addr();
        self.slots.extend(items);
        pos
    }

    pub fn set_root(&mut self, root: TableAddr) {
        self.root = Some(root);
    }

    pub fn commit<T: Space>(self, space: &mut T) -> Result<(), TransactError> {
        let Extend {
            start_addr,
            slots,
            root,
        } = self;
        space.add_segment(start_addr, slots.into_slots(), root)?;
        Ok(())
    }
}
