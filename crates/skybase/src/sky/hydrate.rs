use crate::sky::sky_socket::SkySocketReturn;
use crate::sky::spawn_task::LeptosSpawnTask;
use leptos::leptos_dom::log;
use leptos::prelude::{
    Callable, Effect, Get, ReadSignal, Set, StoredValue, UpdateValue, WithValue, WriteSignal,
};
use leptos_use::core::ConnectionReadyState;
use sky_server::shared::remote::RemoteClient;

pub fn wire_socket_responses_to_client(
    socket: SkySocketReturn,
    client: StoredValue<Option<RemoteClient<LeptosSpawnTask>>>,
    set_client_connected: WriteSignal<bool>,
) {
    Effect::new(move |_| {
        if let ConnectionReadyState::Open = socket.state.get() {
            client.update_value(|opt_client| {
                if opt_client.is_none() {
                    log!("set up sky client");
                    let send_socket = move |req| {
                        log!("got request: {:?}", req);
                        socket.sender.try_run(req);
                    };
                    let client = RemoteClient::<LeptosSpawnTask>::connect(send_socket);
                    *opt_client = Some(client);
                    set_client_connected.set(true);
                    log!("sky client stored");
                }
            });
        } else {
            client.update_value(|opt_client| {
                *opt_client = None;
                set_client_connected.set(false);
            })
        }
    });
}
pub fn wire_client_requests_to_socket(
    client: StoredValue<Option<RemoteClient<LeptosSpawnTask>>>,
    socket: SkySocketReturn,
) {
    Effect::new(move |_| {
        if let Some(response) = socket.receiver.get() {
            // Pass response to the client.
            client.with_value(|client_opt| {
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

pub fn enable_client_ready(
    socket: SkySocketReturn,
    client_connected: ReadSignal<bool>,
    set_client_ready: WriteSignal<bool>,
) {
    Effect::new(move |_| {
        if let (ConnectionReadyState::Open, true) = (socket.state.get(), client_connected.get()) {
            set_client_ready.set(true);
        }
    });
}
