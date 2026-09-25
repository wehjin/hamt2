use crate::storage::file::internal;
use crate::storage::file::internal::{read_max_id_file, write_base};
use crate::storage::{FileTrieView, ReadStorageError, WriteStorageError};
use crate::trie::BaseEdit;
use crate::trie::BaseView;
use crate::trie::{Base, BaseCommit, BaseId, BaseRead, MapBase, TrieStream};
use internal::{read_root_file, write_max_id_file, write_root_file};
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug)]
pub struct FileTrieEdit {
    pub(crate) inner: FileTrieView,
    max_id_path: PathBuf,
    root_path: PathBuf,
}

impl FileTrieEdit {
    pub(crate) const MAX_ID_FILE: &'static str = "max_id";
    pub(crate) const ROOT_FILE: &'static str = "root";

    pub async fn new(path: impl AsRef<Path>) -> Result<Self, WriteStorageError> {
        let frame_dir = path.as_ref();
        fs::create_dir_all(frame_dir)
            .await
            .map_err(|e| WriteStorageError::Io("frame".to_string(), e))?;
        let max_id_path = frame_dir.join(Self::MAX_ID_FILE);
        let root_path = frame_dir.join(Self::ROOT_FILE);
        let inner = FileTrieView::empty(path).await;
        write_max_id_file(&max_id_path, inner.max_id()).await?;
        write_root_file(&root_path, &inner.read_root()).await?;
        Ok(Self {
            inner,
            max_id_path,
            root_path,
        })
    }

    pub async fn load(path: impl AsRef<Path>) -> Result<Self, ReadStorageError> {
        let frame_dir = path.as_ref();
        let max_id_path = frame_dir.join(Self::MAX_ID_FILE);
        let root_path = frame_dir.join(Self::ROOT_FILE);
        let max_id = read_max_id_file(&max_id_path).await?;
        let root = read_root_file(&root_path).await?;
        let inner = FileTrieView::load(frame_dir, max_id, root);
        Ok(Self {
            inner,
            max_id_path,
            root_path,
        })
    }
}

impl BaseEdit for FileTrieEdit {}

impl BaseCommit for FileTrieEdit {
    async fn commit_root(&mut self, root: MapBase) -> Result<(), WriteStorageError> {
        write_root_file(&self.root_path, &root).await?;
        self.inner.root = root;
        Ok(())
    }

    async fn commit_base(&mut self, base: Base) -> Result<BaseId, WriteStorageError> {
        let id = self.next_id();
        write_base(&self.inner.bases_dir, id, base).await?;
        write_max_id_file(&self.max_id_path, id).await?;
        self.inner.max_id = id;
        Ok(id)
    }
}

impl TrieStream for FileTrieEdit {
    type Subtrie = FileTrieView;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.inner.to_subtrie(subtrie_root)
    }
}

impl BaseView for FileTrieEdit {
    type Snapshot = FileTrieView;

    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let inner = self.inner.with_new_root(new_root);
        Self { inner, ..self }
    }

    fn snapshot(&self) -> Self::Snapshot {
        self.inner.snapshot()
    }
}

impl BaseRead for FileTrieEdit {
    fn max_id(&self) -> BaseId {
        self.inner.max_id()
    }

    fn read_root(&self) -> MapBase {
        self.inner.read_root()
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        self.inner.read_base(id).await
    }
}
