use codee::string::FromToStringCodec;
use leptos::logging::error;
use leptos::prelude::*;
use leptos_use::{UseWebSocketReturn, use_websocket};
use sky_server::shared::{SocketRequest, SocketResponse};
use sky_types::db::{Attr, Datom, datom, val};
use sky_types::trie::SlotBaseId;
use std::sync::Arc;

#[derive(Clone)]
pub struct RequestSender {
    send: Arc<dyn Fn(&String)>,
}

impl RequestSender {
    pub fn new(send: Arc<dyn Fn(&String)>) -> Self {
        Self { send }
    }
    fn send_request(&self, request: SocketRequest) {
        let message = serde_json::to_string(&request).expect("serialize request");
        (self.send)(&message)
    }
    pub fn send_connect(&self) {
        self.send_request(SocketRequest::Connect);
    }
    pub fn send_read(&self, id: SlotBaseId) {
        self.send_request(SocketRequest::ReadSlotBase(id));
    }
    pub fn send_transact(&self, datoms: impl Into<Vec<Datom>>) {
        self.send_request(SocketRequest::Transact(datoms.into()));
    }
}

#[component]
pub fn WebSocketSandbox() -> impl IntoView {
    let UseWebSocketReturn {
        ready_state,
        message,
        send,
        ..
    } = use_websocket::<String, String, FromToStringCodec>("/ws");
    let request_sender = RequestSender::new(Arc::new(send.clone()));
    let received_response = Memo::new(move |_| match message.get() {
        None => None,
        Some(json) => {
            let result = serde_json::from_str::<SocketResponse>(&json);
            match result {
                Ok(response) => Some(response),
                Err(e) => {
                    error!("message parse error: {:?}", e);
                    None
                }
            }
        }
    });
    let max_id = Memo::new(move |_| match received_response.get() {
        Some(response) => match response {
            SocketResponse::StorageStatus(head) | SocketResponse::TransactResult(head) => {
                Some(head.max_id)
            }
            _ => None,
        },
        None => None,
    });
    let last_response = Memo::new(move |_| {
        received_response
            .get()
            .map(|response| serde_json::to_string_pretty(&response).expect("serialize response"))
    });
    let send_connect = {
        let sender = request_sender.clone();
        move |_| sender.send_connect()
    };
    let send_read_slot_base = {
        let sender = request_sender.clone();
        move |_| {
            if let Some(id) = max_id.get() {
                sender.send_read(id);
            }
        }
    };

    let send_transact = {
        let sender = request_sender.clone();
        move |_| {
            let datoms = vec![datom::add(100, Attr::from("skybase/version"), val("0.1"))];
            sender.send_transact(datoms);
        }
    };

    view! {
        <section class="box">
            <h1 class="title is-4">"Play with the web socket"</h1>
            <div class="buttons">
                <button class="button" on:click=send_connect>"Connect"</button>
                <button class="button" on:click=send_read_slot_base>"Read slot base"</button>
                <button class="button" on:click=send_transact>"Transact"</button>
            </div>
            <p>
                "Ready state: "
                <strong>{move || format!("{:?}", ready_state.get())}</strong>
                {move || format!(" · max_id: {:?}", max_id.get())}
            </p>
            <pre class="is-size-7">{move || last_response.get()}</pre>
        </section>
    }
}
