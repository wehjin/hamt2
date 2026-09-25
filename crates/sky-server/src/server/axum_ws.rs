//! Web-socket transport glue for [`process_socket_requests`].
//!
//! Hosts mount a websocket route (e.g. in an axum `Router`) and upgrade each
//! connection into [`serve_websocket`], handing off to the shared
//! [`StorageService`].

use crate::server::process_socket_requests;
use crate::server::storage::StorageService;
use crate::shared::protocol::{SocketRequest, SocketResponse};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};

/// Serves a single upgraded websocket connection. Text frames carry JSON
/// [`SocketRequest`]s; responses are sent as JSON [`SocketResponse`]s.
pub async fn serve_websocket(socket: WebSocket, storage: StorageService) {
    let (sink, stream) = socket.split();
    let requests = Box::pin(stream.filter_map(|message| async move {
        match message {
            Ok(message) => request_from_message(message),
            Err(_) => None,
        }
    }));
    let responses = Box::pin(sink.with(|response: SocketResponse| async move {
        Ok::<_, axum::Error>(response_to_message(response))
    }));
    process_socket_requests(requests, responses, storage).await;
}

/// Maps a websocket message onto a socket request. Non-text or malformed
/// messages produce `None` and are skipped.
fn request_from_message(message: Message) -> Option<SocketRequest> {
    match message {
        Message::Text(text) => serde_json::from_str(text.as_str()).ok(),
        _ => None,
    }
}

fn response_to_message(response: SocketResponse) -> Message {
    let json = serde_json::to_string(&response).expect("serialize response");
    Message::Text(json.into())
}

#[cfg(test)]
mod tests {
	use super::*;
	use sky_types::trie::BaseId;

	#[test]
    fn text_messages_map_to_requests() {
        let connect = Message::Text("{\"Connect\":null}".into());
        assert_eq!(Some(SocketRequest::Connect), request_from_message(connect));
    }

    #[test]
    fn malformed_messages_map_to_none() {
        let not_json = Message::Text("hello".into());
        assert_eq!(None, request_from_message(not_json));
        let wrong_shape = Message::Text("{\"Transact\":\"wrong\"}".into());
        assert_eq!(None, request_from_message(wrong_shape));
        let binary = Message::Binary(vec![1, 2, 3].into());
        assert_eq!(None, request_from_message(binary));
    }

    #[test]
    fn responses_map_to_text_messages() {
        let message = response_to_message(SocketResponse::SlotBase(BaseId::ZERO, None));
        let Message::Text(text) = message else {
            panic!("expected text message");
        };
        assert!(serde_json::from_str::<SocketResponse>(text.as_str()).is_ok());
    }
}
