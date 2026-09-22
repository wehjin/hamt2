use crate::shared::SocketRequest;
use crate::shared::remote::requests::ClientRequest;
use sky_types::db::DbStatus;
use sky_types::trie::{Base, BaseId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot;

pub async fn process_client_requests(
    mut recv_request: Receiver<ClientRequest>,
    task_send_socket: Arc<impl Fn(SocketRequest)>,
    db_status: Arc<RwLock<DbStatus>>,
) {
    let mut read_line: HashMap<BaseId, Vec<oneshot::Sender<Option<Base>>>> = HashMap::new();
    let mut bases = HashMap::from([(BaseId::ZERO, Base::empty())]);
    let mut transact_line: Option<oneshot::Sender<Option<DbStatus>>> = None;
    loop {
        let event = recv_request.recv().await;
        if let Some(request) = event {
            match request {
                ClientRequest::Reconnect => task_send_socket(SocketRequest::Connect),
                ClientRequest::DeliverStatus(new_status) => {
                    if new_status.head.max_id > db_status.read().unwrap().head.max_id {
                        let mut write_lock = db_status.write().unwrap();
                        *write_lock = new_status
                    }
                }
                ClientRequest::RequestBase(id, send_base) => {
                    if id > db_status.read().unwrap().head.max_id {
                        let _ = send_base.send(None);
                    } else {
                        if let Some(base) = bases.get(&id).cloned() {
                            let _ = send_base.send(Some(base));
                        } else {
                            let mut line = read_line.remove(&id).unwrap_or_default();
                            line.push(send_base);
                            read_line.insert(id, line);
                            task_send_socket(SocketRequest::ReadSlotBase(id));
                        }
                    }
                }
                ClientRequest::DeliverBase(id, base) => {
                    let line = read_line.remove(&id).unwrap_or_default();
                    for send_base in line {
                        let base = base.clone();
                        let _ = send_base.send(base);
                    }
                    if let Some(base) = base.clone() {
                        bases.insert(id, base);
                    }
                }
                ClientRequest::RequestTransact(datoms, send_status) => {
                    if transact_line.is_some() {
                        let _ = send_status.send(None);
                    } else {
                        transact_line = Some(send_status);
                        task_send_socket(SocketRequest::Transact(datoms));
                    }
                }
                ClientRequest::DeliverTransact(new_head) => {
                    if let Some(send_head) = transact_line.take() {
                        let _ = send_head.send(Some(new_head));
                    }
                }
            }
        }
    }
}
