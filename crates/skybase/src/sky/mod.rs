use crate::sky::sky_client::SkyClient;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
mod hydrate;
mod sky_client;
mod sky_socket;
mod socket_sender;
mod spawn_task;

use crate::sky::sky_socket::use_sky_socket;

#[allow(unused_variables)]
pub fn use_sky() -> SkyClient {
    let socket = use_sky_socket();
    let client = StoredValue::new(None);
    let (client_connected, set_client_connected) = signal(false);
    let (ready, set_ready) = signal(false);
    #[cfg(feature = "hydrate")]
    {
        use crate::sky::hydrate::{
            enable_client_ready, enable_client_requests, enable_socket_responses,
        };
        enable_socket_responses(socket.clone(), client.clone(), set_client_connected);
        enable_client_requests(client.clone(), socket.clone());
        enable_client_ready(socket.clone(), client_connected, set_ready);
    }
    SkyClient {
        client,
        ready,
        socket,
    }
}
