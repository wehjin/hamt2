use crate::TransactError;
use crate::db::{Datom, Db};
use crate::trie::base_storage::BaseStorageReadWrite;
use log::error;
use thiserror::Error;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, oneshot};

#[derive(Error, Debug)]
pub enum HandleError {
    #[error("Task closed {0}")]
    TaskClosed(anyhow::Error),
    #[error("Task failed {0}")]
    TaskFailed(anyhow::Error),
    #[error("Transact failed {0}")]
    TransactFailed(#[from] TransactError),
}

#[derive(Debug, Clone)]
pub struct DbHandle {
    sender: mpsc::Sender<WorkerCommand>,
}

impl DbHandle {
    pub async fn new<S: BaseStorageReadWrite + Send + Sync + 'static>(db: Db<S>) -> Self {
        let (sender, receiver) = mpsc::channel::<WorkerCommand>(32);
        tokio::spawn(async move {
            run_worker(db, receiver).await;
        });
        DbHandle { sender }
    }

    pub async fn transact(&self, datoms: impl Into<Vec<Datom>>) -> Result<(), HandleError> {
        let datoms = datoms.into();
        let (tx, rx) = oneshot::channel::<()>();
        let msg = WorkerCommand::Transact(datoms, tx);
        self.sender
            .send(msg)
            .await
            .map_err(|e| HandleError::TaskClosed(e.into()))?;
        rx.await.map_err(|e| HandleError::TaskFailed(e.into()))?;
        Ok(())
    }
}

enum WorkerCommand {
    Transact(Vec<Datom>, oneshot::Sender<()>),
}

async fn run_worker<S: BaseStorageReadWrite + Send + Sync + 'static>(
    db: Db<S>,
    mut receiver: Receiver<WorkerCommand>,
) {
    let mut db = Some(db);
    while let Some(cmd) = receiver.recv().await {
        match db.take() {
            None => break,
            Some(current) => match cmd {
                WorkerCommand::Transact(datoms, response) => match current.transact(datoms).await {
                    Ok(next_db) => {
                        db = Some(next_db);
                        let _ = response.send(());
                    }
                    Err(e) => error!("Transact failed: {}", e),
                },
            },
        }
        if db.is_none() {
            break;
        }
    }
}
