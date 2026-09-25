use crate::sky::{LeptosSpawnTask, SocketSender};
use leptos::prelude::{Memo, ReadSignal, Signal, StoredValue, WithValue};
use leptos_use::core::ConnectionReadyState;
use sky_db::reader::DbReader;
use sky_server::shared::protocol::SocketResponse;
use sky_server::shared::remote::{Remote, RemoteClient};
use sky_types::db::Datom;

#[derive(Clone)]
pub struct SkyClient {
    pub(crate) stored_client: StoredValue<Option<RemoteClient<LeptosSpawnTask>>>,
    pub ready: ReadSignal<bool>,
    pub socket_sender: SocketSender,
    pub socket_receiver: Memo<Option<SocketResponse>>,
    pub socket_ready: Signal<ConnectionReadyState>,
}

impl SkyClient {
    pub fn reconnect(&self) {
        self.stored_client.with_value(|client_opt| {
            if let Some(client) = client_opt {
                client.reconnect();
            }
        });
    }

    pub fn transact(&self, datoms: impl Into<Vec<Datom>>) {
        self.stored_client.with_value(|client_opt| {
            if let Some(client) = client_opt {
                let _ = client.send_transact(datoms);
            }
        })
    }

    pub fn to_reader(&self) -> Option<DbReader<Remote<LeptosSpawnTask>>> {
        self.stored_client.with_value(|client_opt| {
            if let Some(client) = client_opt {
                Some(client.to_reader())
            } else {
                None
            }
        })
    }
}
