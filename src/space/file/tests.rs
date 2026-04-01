use crate::space::core::reader::SlotValue;
use crate::space::file::FileSpace;
use crate::space::{Read, Space, TableAddr};

#[tokio::test]
async fn high_bits_work() {
    let file = tempfile::NamedTempFile::new().expect("tempfile");
    {
        let mut space = FileSpace::new(&file).await.expect("create red space");
        assert_eq!(TableAddr::ZERO, space.max_addr());
        let mut extend = space.extend().await.unwrap();
        extend.add_slots(vec![
            SlotValue::from((0u32, 0u32)),
            SlotValue::from((0u32, 0x8000_0000u32)),
            SlotValue::from((0x8000_0000u32, 0u32)),
            SlotValue::from((0x8000_0000u32, 0x8000_0000u32)),
        ]);
        extend.commit(&mut space).await.unwrap();
    }
    {
        let space = FileSpace::load(&file).await.expect("load red space");
        let reader = space.read().await.expect("read red space");
        assert_eq!(
            SlotValue::from((0u32, 0u32)),
            reader
                .read_slot(&TableAddr::ZERO, 0)
                .await
                .expect("read slot")
        );
        assert_eq!(
            SlotValue::from((0u32, 0x8000_0000u32)),
            reader
                .read_slot(&TableAddr::ZERO, 1)
                .await
                .expect("read slot")
        );
        assert_eq!(
            SlotValue::from((0x8000_0000u32, 0u32)),
            reader
                .read_slot(&TableAddr::ZERO, 2)
                .await
                .expect("read slot")
        );
        assert_eq!(
            SlotValue::from((0x8000_0000u32, 0x8000_0000u32)),
            reader
                .read_slot(&TableAddr::ZERO, 3)
                .await
                .expect("read slot")
        );
    }
}

#[tokio::test]
async fn file_space_works() {
    let file = tempfile::NamedTempFile::new().expect("tempfile");
    {
        let mut space = FileSpace::new(&file).await.expect("create red space");
        assert_eq!(TableAddr::ZERO, space.max_addr());
        for count in 1..=3 {
            let mut extend = space.extend().await.unwrap();
            for subcount in 0..count {
                let slot = SlotValue::from((count, subcount));
                extend.add_slots(vec![slot]);
            }
            extend.commit(&mut space).await.unwrap();
        }
        assert_eq!(TableAddr::from(6usize), space.max_addr());
    }
    {
        let space = FileSpace::load(&file).await.expect("load red space");
        let reader = space.read().await.expect("read red space");
        let mut slots = Vec::new();
        let mut addr = TableAddr::ZERO;
        for count in 1..=3 {
            for offset in 0..count {
                let slot = reader.read_slot(&addr, offset).await.expect("read slot");
                slots.push(slot);
            }
            addr += count;
        }
        assert_eq!(addr, space.max_addr());
        assert_eq!(
            vec![
                SlotValue::from((1, 0)),
                SlotValue::from((2, 0)),
                SlotValue::from((2, 1)),
                SlotValue::from((3, 0)),
                SlotValue::from((3, 1)),
                SlotValue::from((3, 2)),
            ],
            slots
        );
    }
}
