pub use core::extend::Extend;
use std::fmt::Debug;

use crate::error::ReadError;
pub use core::addr::*;

pub mod core;
pub mod doc;
pub mod file;
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

