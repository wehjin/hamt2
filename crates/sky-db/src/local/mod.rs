use crate::db::Db;
use sky_types::db::schema::attr_spec::DbSpec;
use sky_types::storage::{MemStorage, ReadWriteStorage};

pub trait DbRuntime {
    fn block_on<T>(main: impl Future<Output = T> + 'static) -> T;
}

pub struct LocalDb<S: ReadWriteStorage> {
    _db: Db<S>,
}

pub fn new_in_memory<R: DbRuntime>(db_spec: impl Into<DbSpec>) -> LocalDb<MemStorage> {
    let mem_storage = MemStorage::new();
    let db_spec = db_spec.into();
    let Ok(db) = R::block_on(async move { Db::new(mem_storage, db_spec).await }) else {
        unreachable!("Building db from a mem-storage should not fail")
    };
    LocalDb { _db: db }
}

#[cfg(test)]
mod tests {
    use crate::local::{DbRuntime, new_in_memory};
    use tokio::runtime::Runtime;

    #[test]
    fn in_memory_works() {
        let _mem = new_in_memory::<TokioDbRuntime>(["counter/counter"]);
        assert!(true)
    }

    struct TokioDbRuntime;
    impl DbRuntime for TokioDbRuntime {
        fn block_on<T>(main: impl Future<Output = T> + 'static) -> T {
            Runtime::new().unwrap().block_on(main)
        }
    }
}
