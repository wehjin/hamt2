use crate::LoadError;
use crate::db::Db;
use crate::reader::DbReader;
use sky_types::db::Transact;
use crate::trie::prelude::*;
use log::error;
use sky_types::db::Datom;
use sky_types::db::TransactError;
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
    #[error("Load failed {0}")]
    LoadFailed(#[from] LoadError),
}

pub struct DbHandle<S>
where
    S: ReadWriteTrieStorage + Send + Sync,
{
    sender: mpsc::Sender<WorkerCommand<S>>,
}

impl<S> Clone for DbHandle<S>
where
    S: ReadWriteTrieStorage + Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<S> DbHandle<S>
where
    S: ReadWriteTrieStorage + Send + Sync + 'static,
{
    pub async fn new(db: Db<S>) -> Self {
        let (sender, receiver) = mpsc::channel::<WorkerCommand<S>>(32);
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

    /// Returns a read-only snapshot of the db's current state.
    pub async fn to_reader(&self) -> Result<DbReader<S::Snapshot>, HandleError> {
        let (tx, rx) = oneshot::channel::<Result<DbReader<S::Snapshot>, LoadError>>();
        let msg = WorkerCommand::Reader(tx);
        self.sender
            .send(msg)
            .await
            .map_err(|e| HandleError::TaskClosed(e.into()))?;
        let result = rx.await.map_err(|e| HandleError::TaskFailed(e.into()))?;
        result.map_err(Into::into)
    }
}

impl<S> std::fmt::Debug for DbHandle<S>
where
    S: ReadWriteTrieStorage + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DbHandle")
            .field("sender", &self.sender)
            .finish()
    }
}

enum WorkerCommand<S: ReadWriteTrieStorage> {
    Transact(Vec<Datom>, oneshot::Sender<()>),
    Reader(oneshot::Sender<Result<DbReader<S::Snapshot>, LoadError>>),
}

async fn run_worker<S: ReadWriteTrieStorage + Send + Sync + 'static>(
    db: Db<S>,
    mut receiver: Receiver<WorkerCommand<S>>,
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
                WorkerCommand::Reader(response) => {
                    let result = DbReader::load(&current).await;
                    db = Some(current);
                    let _ = response.send(result);
                }
            },
        }
        if db.is_none() {
            break;
        }
    }
}
