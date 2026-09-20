use crate::storage::{
    FileReadStorage, FileStorage, MemReadStorage, MemStorage, ReadStorageError, StorageHead,
    WriteStorageError,
};
use crate::trie::{
    HandleTrieConfig, MapBase, SlotBase, SlotBaseId, TrieConfig, TrieInsertError, TrieQueryError,
    TrieBaseRead, TrieBaseCommit,
};

/// A trait for reading Bases from storage.
///
/// Base id [`SlotBaseId::ZERO`] is reserved and always represents the empty base.
#[allow(async_fn_in_trait)]
pub trait ReadStorage: TrieBaseRead + Sync {
    /// The storage type of an owned read-only snapshot, produced by
    /// [`ReadStorage::snapshot`]. Writer storages use their read-only
    /// snapshot type; read-only snapshot types usually use `Self`.
    type Snapshot: ReadStorage + TrieBaseRead<Config = Self::Config> + Send + Clone;

    /// Returns an owned read-only snapshot of this storage. The snapshot does
    /// not observe writes made after this call.
    fn snapshot(&self) -> Self::Snapshot;

    /// Reads a base from storage.
    ///
    /// Base id [`SlotBaseId::ZERO`] always reads back the empty base. Reading
    /// any other unwritten id is a programming error: writer storages panic
    /// for ids they have never assigned, and snapshot readers panic for ids
    /// beyond their captured `max_id`. Ids read from committed map bases are
    /// always valid.
    async fn read(
        &self,
        id: <Self::Config as TrieConfig>::HandleType,
    ) -> Result<SlotBase<Self::Config>, ReadStorageError>;

    /// Returns the highest base id in the storage. The empty base id
    /// ([`SlotBaseId::ZERO`]) counts, so an empty storage returns
    /// [`SlotBaseId::ZERO`].
    fn max_id(&self) -> SlotBaseId;

    /// Reads the committed root map base, returning [`MapBase::empty()`] when
    /// no root has been committed yet. Every implementation holds the root in
    /// memory, so this never touches the backing medium.
    fn read_root(&self) -> MapBase<Self::Config>;

    fn get_head(&self) -> StorageHead<Self::Config> {
        let max_id = self.max_id();
        let root = self.read_root();
        StorageHead { max_id, root }
    }
}

macro_rules! impl_handle_trie_read_policy {
    ($storage:ty) => {
        impl TrieBaseRead for $storage {
            type Config = HandleTrieConfig;
            async fn read_base(
                &self,
                id: <HandleTrieConfig as TrieConfig>::HandleType,
            ) -> Result<SlotBase<HandleTrieConfig>, TrieQueryError> {
                let base = self.read(id).await?;
                Ok(base)
            }
        }
    };
}

impl_handle_trie_read_policy!(MemStorage);
impl_handle_trie_read_policy!(MemReadStorage);
impl_handle_trie_read_policy!(FileStorage);
impl_handle_trie_read_policy!(FileReadStorage);

/// A trait for reading and writing Bases from storage.
#[allow(async_fn_in_trait)]
pub trait ReadWriteStorage: ReadStorage {
    /// Read the next available base id. The value is 1 in an empty storage because base id 0 is reserved for the empty base.
    fn next_id(&self) -> SlotBaseId {
        self.max_id() + 1
    }

    /// Stores a base and assigns it the next available id. The id can be used to read back the base in `BaseStorageRead::read`.
    async fn append(
        &mut self,
        base: &SlotBase<Self::Config>,
    ) -> Result<<Self::Config as TrieConfig>::HandleType, WriteStorageError>;

    /// Persists the given root map base. The root can be read back with `BaseStorageRead::read_root`.
    async fn write_root(&mut self, root: MapBase<Self::Config>) -> Result<(), WriteStorageError>;
}

impl<T> TrieBaseCommit for T
where
    T: ReadWriteStorage,
{
    async fn commit_base(
        &mut self,
        base: SlotBase<Self::Config>,
    ) -> Result<<Self::Config as TrieConfig>::HandleType, TrieInsertError> {
        let id = self.append(&base).await?;
        Ok(id)
    }
}
