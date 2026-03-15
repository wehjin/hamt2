use crate::base::Change;
use crate::client::{Client, TransactError};

impl Client {
    pub fn transact(&mut self, _changes: &[Change]) -> Result<(), TransactError> {
        Ok(())
    }
}
