use crate::sky;
use leptos::logging::log;
use leptos::prelude::*;
use sky_db::traits::DbQuery;
use sky_server::shared::protocol::SocketResponse;
use sky_types::db;
use sky_types::db::{Attr, datom, val};

#[component]
pub fn WebSocketSandbox() -> impl IntoView {
    let sky = sky::use_sky();
    {
        let sky = sky.clone();
        Effect::new(move |_| {
            if sky.ready.get() {
                sky.reconnect();
            }
        });
    }
    let local = {
        let sky = sky.clone();
        LocalResource::new(move || {
            let sky = sky.clone();
            async move {
                let reader = sky.to_reader();
                if let Some(reader) = reader {
                    let val = reader.find_val(2, db::ident()).await;
                    log!("found: {:?}", val);
                }
            }
        })
    };

    let max_id = {
        let socket_receiver = sky.socket.receiver.clone();
        Memo::new(move |_| match socket_receiver.get() {
            Some(response) => match response {
                SocketResponse::DbStatus(status) | SocketResponse::TransactResult(status) => {
                    Some(status.head.max_id)
                }
                _ => None,
            },
            None => None,
        })
    };
    let last_response = {
        let socket_receiver = sky.socket.receiver.clone();
        Memo::new(move |_| {
            socket_receiver.get().map(|response| {
                serde_json::to_string_pretty(&response).expect("serialize response")
            })
        })
    };
    let send_connect = {
        let sky = sky.clone();
        move |_| sky.reconnect()
    };
    let send_read_slot_base = {
        let sender = sky.socket.sender.clone().clone();
        move |_| {
            if let Some(id) = max_id.get() {
                sender.send_read(id);
            }
        }
    };
    let read_ident = {
        let local = local.clone();
        move |_| {
            local.refetch();
        }
    };

    let send_transact = {
        let sky = sky.clone();
        move |_| {
            let datom = datom::add(100, Attr::from("skybase/version"), val("0.1"));
            sky.transact(vec![datom]);
        }
    };

    let socket_ready = sky.socket.state.clone();
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
                <strong>{move || format!("{:?}", socket_ready.get())}</strong>
                {move || format!(" · max_id: {:?}", max_id.get())}
            </p>
            <pre class="is-size-7">{move || last_response.get()}</pre>
        </section>
    }
}
