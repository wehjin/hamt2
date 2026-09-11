use hamt2::db::query::DbQuery;
use hamt2::db::viewer::DbViewer;
use hamt2::db::{Attr, Db, datom, val};
use hamt2::trie::base_storage::mem::MemBaseStorage;
use hamt2::TransactError;
use std::sync::Arc;

pub const ATTR_SKYBASE_VERSION: Attr = Attr("skybase/version");
pub const EID_SKYBASE: i32 = 1;

pub async fn start_db() -> Result<Db<MemBaseStorage>, TransactError> {
    let db = Db::new(MemBaseStorage::new(), [ATTR_SKYBASE_VERSION]).await?;
    db.transact([datom::add(EID_SKYBASE, ATTR_SKYBASE_VERSION, val("0.1"))])
        .await
}

#[derive(Clone)]
pub struct SkyViewer(Arc<DbViewer<MemBaseStorage>>);

impl SkyViewer {
    pub fn start(storage: MemBaseStorage) -> SkyViewer {
        let viewer = pollster::block_on(async move {
            DbViewer::load(storage, [ATTR_SKYBASE_VERSION])
                .await
                .expect("load viewer failed")
        });
        SkyViewer(Arc::new(viewer))
    }

    pub fn get_version(&self) -> String {
        let version =
            pollster::block_on(
                async move { self.0.get_val(EID_SKYBASE, ATTR_SKYBASE_VERSION).await },
            );
        version.as_str().to_string()
    }
}
