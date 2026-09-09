use hamt2::db::{Attr, Db, datom, val};
use hamt2::space::mem::MemSpace;
use hamt2::{QueryError, TransactError};
use std::sync::mpsc::{Sender, channel};
use std::thread;
use thiserror::Error;

pub const ATTR_SKYBASE_VERSION: Attr = Attr("skybase/version");
pub const ENT_SKYBASE: i32 = 1;

#[derive(Debug, Clone)]
pub struct SkyDb {
    sender: Sender<ConnEvent>,
}

impl SkyDb {
    pub fn connect() -> Result<Self, ConnectError> {
        connect()
    }

    pub fn version(&self) -> Result<String, ConnectError> {
        let (send, receive) = channel::<String>();
        let event = ConnEvent::Version(send);
        if let Err(_) = self.sender.send(event) {
            return Err(ConnectError::Closed);
        }
        match receive.recv() {
            Ok(answer) => Ok(answer),
            Err(_) => Err(ConnectError::Closed),
        }
    }
}

#[derive(Error, Debug)]
pub enum ConnectError {
    #[error("Runtime error: {0}")]
    RuntimeError(#[from] std::io::Error),
    #[error("Connection closed")]
    Closed,
}

#[derive(Debug)]
enum ConnEvent {
    Version(Sender<String>),
}

fn connect() -> Result<SkyDb, ConnectError> {
    let (sender, receiver) = channel::<ConnEvent>();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    thread::spawn(move || {
        rt.block_on(async move {
            enum State {
                TransactError(TransactError),
                QueryError(QueryError),
                Running(Db<MemSpace>),
            }
            let mut state = match Db::new(MemSpace::new(), [ATTR_SKYBASE_VERSION]).await {
                Ok(db) => State::Running(db),
                Err(err) => State::TransactError(err),
            };
            if let State::Running(db) = state {
                state = match db
                    .transact([datom::add(ENT_SKYBASE, ATTR_SKYBASE_VERSION, val("0.1"))])
                    .await
                {
                    Ok(db) => State::Running(db),
                    Err(err) => State::TransactError(err),
                }
            };
            for conn_event in receiver.iter() {
                match state {
                    State::TransactError(err) => {
                        eprintln!("Db connection error: {}", err);
                        break;
                    }
                    State::QueryError(err) => {
                        eprintln!("Db query error: {}", err);
                        break;
                    }
                    State::Running(db) => match conn_event {
                        ConnEvent::Version(responder) => {
                            let result = db.find_val(ENT_SKYBASE, ATTR_SKYBASE_VERSION).await;
                            match result {
                                Ok(option) => {
                                    state = State::Running(db);
                                    let version =
                                        option.expect("missing version").as_str().to_string();
                                    let _ = responder.send(version);
                                }
                                Err(err) => {
                                    state = State::QueryError(err);
                                }
                            }
                        }
                    },
                }
            }
        })
    });
    Ok(SkyDb { sender })
}
