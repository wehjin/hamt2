use codee::string::FromToStringCodec;
use leptos::prelude::*;
use leptos_use::{use_websocket, UseWebSocketReturn};
use sky_server::shared::{SocketRequest, SocketResponse};
use sky_types::db::{Attr, datom, val};
use sky_types::trie::SlotBaseId;

#[component]
pub fn WebSocketSandbox() -> impl IntoView {
    let UseWebSocketReturn {
        ready_state,
        message,
        send,
        ..
    } = use_websocket::<String, String, FromToStringCodec>("/ws");

    let max_id = RwSignal::new(None::<SlotBaseId>);
    let last_response = RwSignal::new(String::new());

    Effect::new(move || {
        let Some(json) = message.get() else {
            return;
        };
        match serde_json::from_str::<SocketResponse>(&json) {
            Ok(response) => {
                last_response.set(
                    serde_json::to_string_pretty(&response).expect("serialize response"),
                );
                match response {
                    SocketResponse::StorageStatus(head)
                    | SocketResponse::TransactResult(head) => {
                        max_id.set(Some(head.max_id));
                    }
                    _ => {}
                }
            }
            Err(e) => last_response.set(format!("unexpected message: {json:?} ({e})")),
        }
    });

    let send_request = {
        let send = send.clone();
        move |request: SocketRequest| {
            send(&serde_json::to_string(&request).expect("serialize request"));
        }
    };

    let send_connect = {
        let send_request = send_request.clone();
        move |_| send_request(SocketRequest::Connect)
    };
    let send_read_slot_base = {
        let send_request = send_request.clone();
        move |_| {
            if let Some(id) = max_id.get() {
                send_request(SocketRequest::ReadSlotBase(id));
            }
        }
    };
    let send_transact = move |_| {
        send_request(SocketRequest::Transact(vec![datom::add(
            100,
            Attr::from("skybase/version"),
            val("0.1"),
        )]));
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
