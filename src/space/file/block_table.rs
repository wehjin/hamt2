use crate::space::core::reader::SlotValue;
use crate::space::TableAddr;
use redb::TableDefinition;

pub struct BlockTable;

impl BlockTable {
    const BLOCKS_TABLE: TableDefinition<'static, u32, Vec<u64>> = TableDefinition::new("blocks");

    pub fn write_slots(write: &redb::WriteTransaction, start: TableAddr, slots: Vec<SlotValue>) {
        let mut table = write
            .open_table(Self::BLOCKS_TABLE)
            .expect("open blocks table");
        let key = start.u32();
        let value = slots.into_iter().map(|s| s.to_u64()).collect::<Vec<_>>();
        table.insert(key, value).expect("insert slots");
    }

    pub fn read_slots(
        read: &redb::ReadTransaction,
        start: TableAddr,
    ) -> Option<(u32, Vec<SlotValue>)> {
        let table = read
            .open_table(Self::BLOCKS_TABLE)
            .expect("open blocks table");
        let mut range = table.range(..=start.u32()).expect("get range");
        match range.next_back() {
            None => None,
            Some(entry) => {
                let (key_guard, value_guard) = entry.expect("key and value in result");
                let key = key_guard.value();
                let slots = value_guard
                    .value()
                    .into_iter()
                    .map(|v| SlotValue::from_u64(v))
                    .collect();
                Some((key, slots))
            }
        }
    }
}
