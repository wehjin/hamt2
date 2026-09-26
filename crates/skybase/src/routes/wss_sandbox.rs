use crate::sky;
use crate::sky::UseSkyReturn;
use leptos::logging::log;
use leptos::prelude::*;
use sky::use_sky;
use sky_db::traits::DbQuery;
use sky_server::shared::protocol::SocketResponse;
use sky_types::db;
use sky_types::db::{Attr, datom, val};

#[component]
pub fn WebSocketSandbox() -> impl IntoView {
    let UseSkyReturn {
        client,
        client_ready,
        socket,
    } = use_sky();
    let local = LocalResource::new(move || async move {
        if client_ready.get() {
            let reader = client.with_value(|client| {
                if let Some(client) = client {
                    Some(client.to_reader())
                } else {
                    None
                }
            });
            if let Some(reader) = reader {
                let val = reader.find_val(2, db::ident()).await;
                log!("found: {:?}", val);
            }
        }
    });

    let max_id = Memo::new(move |_| match socket.receiver.get() {
        Some(response) => match response {
            SocketResponse::DbStatus(status) | SocketResponse::TransactResult(status) => {
                Some(status.head.max_id)
            }
            _ => None,
        },
        None => None,
    });
    let last_response = {
        let socket_receiver = socket.receiver.clone();
        Memo::new(move |_| {
            socket_receiver.get().map(|response| {
                serde_json::to_string_pretty(&response).expect("serialize response")
            })
        })
    };
    let send_connect = move |_| {
        client.with_value(|client_opt| {
            if let Some(client) = client_opt {
                client.reconnect();
            }
        })
    };
    let send_read_slot_base = move |_| {
        if let Some(id) = max_id.get() {
            socket.sender.send_read(id);
        }
    };
    let read_ident = move |_| local.refetch();

    let send_transact = {
        let sky = use_sky().clone();
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
                <button class="button" on:click=read_ident>"Read Ident"</button>
            </div>
            <p>
                "Ready state: "
                <strong>{move || format!("{:?}", socket.state.get())}</strong>
                {move || format!(" · max_id: {:?}", max_id.get())}
            </p>
            <pre class="is-size-7">{move || last_response.get()}</pre>
        </section>
    }
}
