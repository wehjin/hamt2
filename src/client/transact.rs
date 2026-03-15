use crate::base::Change;
use crate::client::Client;

impl Client {
    pub fn transact(&mut self, _changes: &[Change]) -> Result<(), TransactError> {
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TransactError {}
