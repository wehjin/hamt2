use crate::shared::remote::requests::ClientRequest;
use crate::shared::remote::{RemoteClient, SpawnTask};
use anyhow::anyhow;
use sky_types::db::{Datom, TransactError};

impl<T: SpawnTask> RemoteClient<T> {
    pub async fn transact(self, datoms: impl Into<Vec<Datom>>) -> Result<Self, TransactError>
    where
        Self: Sized,
    {
        match self.send_transact(datoms).await {
            Err(e) => Err(TransactError::Disconnected(e.into())),
            Ok(None) => Err(TransactError::Refused(anyhow!("Transaction failed"))),
            Ok(Some(head)) => {
                self.send_request(ClientRequest::DeliverStatus(head));
                Ok(self)
            }
        }
    }
}
