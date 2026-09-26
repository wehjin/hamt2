use leptos::prelude::*;
use sky_server::shared::remote::RemoteClient;
use sky_types::db::Datom;

#[cfg(feature = "hydrate")]
mod hydrate;
mod sky_socket;
mod socket_sender;
mod spawn_task;

use crate::sky::sky_socket::{SkySocketReturn, use_sky_socket};
use crate::sky::spawn_task::LeptosSpawnTask;

#[allow(unused_variables)]
pub fn use_sky() -> UseSkyReturn {
    let socket = use_sky_socket();
    let client = StoredValue::new(None);
    let (client_connected, set_client_connected) = signal(false);
    let (client_ready, set_ready) = signal(false);
    #[cfg(feature = "hydrate")]
    {
        use crate::sky::hydrate::{
            enable_client_ready, enable_client_requests, enable_socket_responses,
        };
        enable_socket_responses(socket.clone(), client.clone(), set_client_connected);
        enable_client_requests(client.clone(), socket.clone());
        enable_client_ready(socket.clone(), client_connected, set_ready);
    }
    UseSkyReturn {
        client,
        client_ready,
        socket,
    }
}

#[derive(Clone)]
pub struct UseSkyReturn {
    pub client: StoredValue<Option<RemoteClient<LeptosSpawnTask>>>,
    pub client_ready: ReadSignal<bool>,
    pub socket: SkySocketReturn,
}

impl UseSkyReturn {
    pub fn transact(&self, datoms: impl Into<Vec<Datom>>) {
        self.client.with_value(|client_opt| {
            if let Some(client) = client_opt {
                let _ = client.send_transact(datoms);
            }
        })
    }
}
