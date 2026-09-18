use crate::routes::wss_sandbox::sky::SocketSender;
use codee::string::FromToStringCodec;
use leptos::logging::error;
use leptos::prelude::*;
use leptos_use::{UseWebSocketReturn, use_websocket};
use sky_server::shared::SocketResponse;
use sky_types::db::{Attr, datom, val};
use std::sync::Arc;

pub mod sky {
    use leptos::prelude::*;
    use sky_server::shared::remote::{RemoteClient, SpawnTask};
    use sky_server::shared::{SocketRequest, SocketResponse};
    use sky_types::db::Datom;
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

    #[derive(Clone)]
    pub struct LeptosSpawnTask;
    impl SpawnTask for LeptosSpawnTask {
        fn spawn_task(future: impl Future<Output = ()> + 'static) {
            leptos::task::spawn_local(future);
        }
    }

    #[derive(Clone)]
    pub struct SkyClient {
        client: StoredValue<Option<RemoteClient<LeptosSpawnTask>>>,
        pub ready: ReadSignal<bool>,
    }

    impl SkyClient {
        pub fn reconnect(&self) {
            self.client.with_value(|client_opt| {
                if let Some(client) = client_opt {
                    client.reconnect();
                }
            });
        }

        pub fn transact(&self, datoms: impl Into<Vec<Datom>>) {
            self.client.with_value(|client_opt| {
                if let Some(client) = client_opt {
                    let _ = client.send_transact(datoms);
                }
            })
        }
    }

    #[allow(unused_variables)]
    pub fn use_sky(
        socket_sender: SocketSender,
        socket_receiver: Signal<Option<SocketResponse>>,
    ) -> SkyClient {
        let stored_client = StoredValue::new(None);
        let (ready, set_ready) = signal(false);
        #[cfg(feature = "hydrate")]
        {
            use leptos::logging::log;
            // Start the client.
            Effect::new(move |_| {
                stored_client.update_value(|stored_client| {
                    if stored_client.is_none() {
                        log!("set up sky client");
                        let socket_sender = socket_sender.clone();
                        let send_socket = move |req| {
                            log!("got request: {:?}", req);
                            socket_sender.send_request(req);
                        };
                        let client = RemoteClient::<LeptosSpawnTask>::connect(send_socket);
                        *stored_client = Some(client);
                        set_ready.set(true);
                        log!("sky client stored");
                    }
                });
            });
            // Send sockets responses to the client.
            Effect::new(move |_| {
                let response_opt = socket_receiver.get();
                if let Some(response) = response_opt {
                    // Pass response to the client.
                    stored_client.with_value(|client_opt| {
                        if let Some(client) = client_opt {
                            log!("update sky client: {:?}", response);
                            let mut updater = client.to_updater();
                            updater.update(response);
                        } else {
                            log!("no sky client to update: {:?}", response);
                        }
                    });
                }
            });
        }
        SkyClient {
            client: stored_client,
            ready,
        }
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
    let sky = sky::use_sky(socket_sender.clone(), socket_receiver.into());
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
