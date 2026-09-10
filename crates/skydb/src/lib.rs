use hamt2::db::query::DbQuery;
use hamt2::db::viewer::DbViewer;
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
    sender: Sender<DbEvent>,
}

impl SkyDb {
    pub fn connect() -> Result<Self, ConnectError> {
        connect()
    }

    pub fn version(&self) -> Result<String, ConnectError> {
        let (send, receive) = channel::<String>();
        let event = ViewerEvent::Version(send);
        if let Err(_) = self.sender.send(event.into()) {
            return Err(ConnectError::Closed);
        }
        match receive.recv() {
            Ok(answer) => Ok(answer),
            Err(_) => Err(ConnectError::Closed),
        }
    }
    pub fn to_viewer(&self) -> SkyViewer {
        let (send, receive) = channel::<SkyViewer>();
        let event = DbEvent::ToViewer(send);
        self.sender.send(event.into()).expect("send event");
        receive.recv().expect("receive viewer")
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
enum DbEvent {
    ViewerEvent(ViewerEvent),
    ToViewer(Sender<SkyViewer>),
}

impl From<ViewerEvent> for DbEvent {
    fn from(event: ViewerEvent) -> Self {
        Self::ViewerEvent(event)
    }
}
#[derive(Debug)]
enum ViewerEvent {
    Version(Sender<String>),
}

fn connect() -> Result<SkyDb, ConnectError> {
    let (sender, receiver) = channel::<DbEvent>();
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
                        DbEvent::ToViewer(responder) => {
                            let viewer = SkyViewer::start(db.to_viewer());
                            state = State::Running(db);
                            let _ = responder.send(viewer);
                        }
                        DbEvent::ViewerEvent(ViewerEvent::Version(responder)) => {
                            let version = get_version(&db).await;
                            match version {
                                Ok(version) => {
                                    state = State::Running(db);
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

async fn get_version(viewer: &impl DbQuery) -> Result<String, QueryError> {
    let version = viewer
        .find_val(ENT_SKYBASE, ATTR_SKYBASE_VERSION)
        .await?
        .expect("missing version")
        .as_str()
        .to_string();
    Ok(version)
}

#[derive(Clone)]
pub struct SkyViewer {
    sender: Sender<ViewerEvent>,
}

impl SkyViewer {
    pub fn version(&self) -> String {
        let (send, receive) = channel::<String>();
        let event = ViewerEvent::Version(send);
        self.sender.send(event.into()).expect("send event");
        let answer = receive.recv().expect("receive answer");
        answer
    }

    fn start(viewer: DbViewer<MemSpace>) -> SkyViewer {
        let (sender, receiver) = channel::<ViewerEvent>();
        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime");
            rt.block_on(async move {
                for event in receiver.iter() {
                    match event {
                        ViewerEvent::Version(responder) => {
                            let version =
                                get_version(&viewer).await.expect("version must be present");
                            let _ = responder.send(version);
                        }
                    }
                }
            });
        });
        SkyViewer { sender }
    }
}
