pub use core::extend::Extend;
use std::fmt::Debug;

use crate::error::ReadError;
pub use core::addr::*;

pub mod block;
pub mod core;
pub mod file;
pub mod iroh;
pub mod mem;

use crate::TransactError;
pub use core::reader::MemReader;
use core::reader::SlotValue;
pub use core::value::Value;

pub trait Space: Debug + Sized {
    type Reader: Read + Clone + Debug;

    fn add_segment(
        &mut self,
        start_addr: TableAddr,
        slots: Vec<SlotValue>,
        root: Option<TableAddr>,
    ) -> impl Future<Output = Result<(), TransactError>>;
    fn read(&self) -> impl Future<Output = Result<Self::Reader, ReadError>>;
    fn max_addr(&self) -> TableAddr;
    fn extend(&self) -> impl Future<Output = Result<Extend<Self>, TransactError>> {
        async move {
            let reader = self.read().await?;
            Ok(Extend::new(self.max_addr(), reader))
        }
    }
}

pub trait Read {
    fn read_slot(
        &self,
        addr: &TableAddr,
        offset: usize,
    ) -> impl Future<Output = Result<SlotValue, ReadError>>;
    fn read_root(&self) -> impl Future<Output = Result<&Option<TableAddr>, ReadError>>;
}

#[cfg(test)]
mod tests {
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
}
