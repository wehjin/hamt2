use crate::sky;
use crate::sky::SocketSender;
use codee::string::FromToStringCodec;
use leptos::logging::error;
use leptos::prelude::*;
use leptos_use::{UseWebSocketReturn, use_websocket};
use sky_server::shared::SocketResponse;
use sky_types::db::{Attr, datom, val};
use std::sync::Arc;

#[component]
pub fn WebSocketSandbox() -> impl IntoView {
    let UseWebSocketReturn {
        ready_state,
        message,
        send,
        ..
    } = use_websocket::<String, String, FromToStringCodec>("/ws");
    let socket_sender = SocketSender::new(Arc::new(send.clone()));
    let socket_receiver = Memo::new(move |_| match message.get() {
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
    let sky = sky::use_sky(
        socket_sender.clone(),
        socket_receiver.into(),
        ready_state.clone(),
    );
    {
        let sky = sky.clone();
        Effect::new(move |_| {
            if sky.ready.get() {
                sky.reconnect();
            }
        });
    }
    let max_id = Memo::new(move |_| match socket_receiver.get() {
        Some(response) => match response {
            SocketResponse::StorageStatus(head) | SocketResponse::TransactResult(head) => {
                Some(head.max_id)
            }
            _ => None,
        },
        None => None,
    });
    let last_response = Memo::new(move |_| {
        socket_receiver
            .get()
            .map(|response| serde_json::to_string_pretty(&response).expect("serialize response"))
    });
    let send_connect = {
        let sky = sky.clone();
        move |_| sky.reconnect()
    };
    let send_read_slot_base = {
        let sender = socket_sender.clone();
        move |_| {
            if let Some(id) = max_id.get() {
                sender.send_read(id);
            }
        }
    };

    let send_transact = {
        let sky = sky.clone();
        move |_| {
            let datom = datom::add(100, Attr::from("skybase/version"), val("0.1"));
            sky.transact(vec![datom]);
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
