use sky_server::shared::SocketRequest;
use sky_types::trie::SlotBaseId;
use std::sync::Arc;

#[derive(Clone)]
pub struct SocketSender {
    send: Arc<dyn Fn(&String)>,
}

impl SocketSender {
    pub fn new(send: Arc<dyn Fn(&String)>) -> Self {
        Self { send }
    }
    pub fn send_read(&self, id: SlotBaseId) {
        self.send_request(SocketRequest::ReadSlotBase(id));
    }
    pub fn send_request(&self, request: SocketRequest) {
        let message = serde_json::to_string(&request).expect("serialize request");
        (self.send)(&message)
    }
}
