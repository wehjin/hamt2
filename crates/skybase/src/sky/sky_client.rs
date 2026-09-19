use crate::sky::LeptosSpawnTask;
use leptos::prelude::{ReadSignal, StoredValue, WithValue};
use sky_db::reader::DbReader;
use sky_server::shared::remote::{RemoteClient, RemoteClientReadStorage};
use sky_types::db::Datom;

#[derive(Clone)]
pub struct SkyClient {
    pub(crate) stored_client: StoredValue<Option<RemoteClient<LeptosSpawnTask>>>,
    pub ready: ReadSignal<bool>,
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

    pub fn to_reader(&self) -> Option<DbReader<RemoteClientReadStorage<LeptosSpawnTask>>> {
        self.stored_client.with_value(|client_opt| {
            if let Some(client) = client_opt {
                Some(client.to_reader())
            } else {
                None
            }
        })
    }
}
