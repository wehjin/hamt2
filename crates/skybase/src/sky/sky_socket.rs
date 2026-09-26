use codee::string::FromToStringCodec;
use leptos::leptos_dom::error;
use leptos::prelude::{Callback, Get, Memo, Signal};
use leptos_use::core::ConnectionReadyState;
use leptos_use::{UseWebSocketReturn, use_websocket};
use sky_server::shared::protocol::{SocketRequest, SocketResponse};

pub fn use_sky_socket() -> SkySocketReturn {
    let UseWebSocketReturn {
        ready_state: state,
        message,
        send,
        ..
    } = use_websocket::<String, String, FromToStringCodec>("/ws");
    let sender = Callback::new(move |req: SocketRequest| {
        let message = serde_json::to_string(&req).expect("serialize request");
        send(&message);
    });
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
    SkySocketReturn {
        state,
        sender,
        receiver,
    }
}

#[derive(Copy, Clone)]
pub struct SkySocketReturn {
    pub state: Signal<ConnectionReadyState>,
    pub sender: Callback<SocketRequest>,
    pub receiver: Memo<Option<SocketResponse>>,
}
