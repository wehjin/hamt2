use crate::sky::SocketSender;
use codee::string::FromToStringCodec;
use leptos::leptos_dom::error;
use leptos::prelude::{Get, Memo, Signal};
use leptos_use::core::ConnectionReadyState;
use leptos_use::{UseWebSocketReturn, use_websocket};
use sky_server::shared::protocol::SocketResponse;
use std::sync::Arc;

#[derive(Clone)]
pub struct SkySocketReturn {
    pub ready: Signal<ConnectionReadyState>,
    pub sender: SocketSender,
    pub receiver: Memo<Option<SocketResponse>>,
}
pub fn use_sky_socket() -> SkySocketReturn {
    let UseWebSocketReturn {
        ready_state,
        message,
        send,
        ..
    } = use_websocket::<String, String, FromToStringCodec>("/ws");
    let sender = SocketSender::new(Arc::new(send.clone()));
    let receiver = Memo::new(move |_| match message.get() {
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
    let ready = ready_state.clone();
    SkySocketReturn {
        ready,
        sender,
        receiver,
    }
}
