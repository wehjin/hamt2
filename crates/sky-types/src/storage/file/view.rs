use crate::storage::ReadStorageError;
use crate::storage::file::internal::{bases_dir, init_bases_dir_with_empty_base, read_base};
use crate::trie::BaseView;
use crate::trie::{Base, BaseId, BaseRead, MapBase, TrieStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct FileTrieView {
    pub(crate) past: Option<Arc<FileTrieView>>,
    pub(crate) bases_dir: PathBuf,
    pub(crate) max_id: BaseId,
    pub(crate) root: MapBase,
}

impl FileTrieView {
    pub(crate) async fn empty(frame_dir: impl AsRef<Path>) -> Self {
        let bases_dir = init_bases_dir_with_empty_base(frame_dir)
            .await
            .expect("initialize bases dir");
        Self {
            past: None,
            bases_dir,
            max_id: BaseId::ZERO,
            root: MapBase::empty(),
        }
    }
    pub(crate) fn load(frame_dir: impl AsRef<Path>, max_id: BaseId, root: MapBase) -> Self {
        let bases_dir = bases_dir(frame_dir);
        Self {
            past: None,
            bases_dir,
            max_id,
            root,
        }
    }
}

impl TrieStream for FileTrieView {
    type Subtrie = FileTrieView;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.clone().with_new_root(Some(subtrie_root))
    }
}

impl BaseView for FileTrieView {
    type Snapshot = FileTrieView;

    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let root = new_root.unwrap_or(self.root);
        Self { root, ..self }
    }

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }
}

impl BaseRead for FileTrieView {
    fn max_id(&self) -> BaseId {
        self.max_id
    }

    fn read_root(&self) -> MapBase {
        self.root
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        if id <= BaseId::ZERO {
            return Ok(Base::empty());
        }
        let Some(past) = &self.past else {
            // no past
            let base = if id > self.max_id {
                Base::empty()
            } else {
                read_base(&self.bases_dir, id).await?
            };
            return Ok(base);
        };
        // yes past
        if id <= past.max_id {
            Box::pin(past.read_base(id)).await
        } else {
            read_base(&self.bases_dir, id).await
        }
    }
}
