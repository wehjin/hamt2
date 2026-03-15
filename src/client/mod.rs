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
