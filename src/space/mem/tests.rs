use crate::space::core::reader::SlotValue;
use crate::space::mem::MemSpace;
use crate::space::{Read, Space, TableAddr};

#[tokio::test]
async fn mem_space_works() {
    let addr: TableAddr;
    {
        let mut space = MemSpace::new();
        assert_eq!(TableAddr::ZERO, space.max_addr());
        {
            let mut extend = space.extend().await.unwrap();
            let slot = SlotValue::from((1, 2));
            addr = extend.add_slots(vec![slot]);
            extend.commit(&mut space).await.unwrap();
        }
        let reader = space.read().await.unwrap();
        let slot = reader.read_slot(&addr, 0).await.unwrap();
        assert_eq!(SlotValue::from((1, 2)), slot);
    }
}
