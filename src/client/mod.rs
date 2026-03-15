use iroh::endpoint::BindError;
use iroh::protocol::Router;
use iroh::Endpoint;
use iroh_docs::api::Doc;
use iroh_docs::protocol::Docs;
use std::rc::Rc;

pub mod connect;
pub mod transact;

pub struct Client {
    _endpoint: Endpoint,
    _docs: Docs,
    _router: Router,
    _doc: Rc<Doc>,
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Endpoint bind error: {0}")]
    EndpointBindError(#[from] BindError),

    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum TransactError {}
