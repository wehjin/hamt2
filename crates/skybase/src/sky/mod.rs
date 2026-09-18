use leptos::prelude::*;
use leptos_use::core::ConnectionReadyState;
use sky_server::shared::SocketResponse;
mod sky_client;
mod socket_sender;
mod spawn_task;

pub use sky_client::*;
pub use socket_sender::*;
pub use spawn_task::*;

#[allow(unused_variables)]
pub fn use_sky(
    socket_sender: SocketSender,
    socket_receiver: Signal<Option<SocketResponse>>,
    connection_ready: Signal<ConnectionReadyState>,
) -> SkyClient {
    let stored_client = StoredValue::new(None);
    let (client_ready, set_client_ready) = signal(false);
    let (sky_ready, set_sky_ready) = signal(false);
    #[cfg(feature = "hydrate")]
    {
        use leptos::logging::log;
        use sky_server::shared::remote::RemoteClient;
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
                    set_client_ready.set(true);
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
        {
            let ws_ready = connection_ready.clone();
            let client_ready = client_ready.clone();
            Effect::new(move |_| {
                if let (ConnectionReadyState::Open, true) = (ws_ready.get(), client_ready.get()) {
                    set_sky_ready.set(true);
                }
            });
        }
    }
    SkyClient {
        stored_client,
        ready: sky_ready,
    }
}
