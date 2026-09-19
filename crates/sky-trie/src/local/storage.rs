use crate::types::slot_base::SlotBase;
use sky_types::storage::StorageHead;
use sky_types::trie::{MapBase, SlotBaseId};
use std::borrow::Cow;
use std::ops::Index;

#[derive(Clone)]
pub struct LocalStorage<'a> {
    head: StorageHead,
    bases: Cow<'a, [SlotBase]>,
}

impl<'a> Default for LocalStorage<'a> {
    fn default() -> Self {
        let head = StorageHead::default();
        let bases = vec![SlotBase::new()];
        let bases = Cow::from(bases);
        Self { head, bases }
    }
}

impl Index<SlotBaseId> for LocalStorage<'_> {
    type Output = SlotBase;

    fn index(&self, index: SlotBaseId) -> &Self::Output {
        &self.bases[index.0 as usize]
    }
}

impl<'a> LocalStorage<'a> {
    /// The max_id of a fresh storage is SlotBaseId::ZERO and only
    /// ever increases.
    pub fn max_id(&self) -> SlotBaseId {
        self.head.max_id
    }

    /// The root of a fresh storage holds an empty map and an empty base.
    pub fn read_root(&self) -> MapBase {
        self.head.root
    }

    /// Add a base using the next available id.
    pub fn push(self, base: SlotBase) -> (Self, SlotBaseId) {
        let next_id = self.head.max_id + 1;
        let Self {
            mut head,
            mut bases,
        } = self;
        head.max_id = next_id;
        bases.to_mut().push(base);
        (Self { head, bases }, next_id)
    }

    /// Change the stored root
    pub fn write_root(self, root: MapBase) -> Self {
        let Self { mut head, bases } = self;
        head.root = root;
        Self { head, bases }
    }
}

#[cfg(test)]
mod tests {
    use crate::local::storage::LocalStorage;
    use crate::types::slot_base::SlotBase;
    use sky_types::trie::{MapBase, SlotBaseId, SlotMap};

    #[test]
    fn write_root() {
        let root = MapBase {
            map: SlotMap(5),
            base: SlotBaseId(1),
        };
        let storage = LocalStorage::default().write_root(root.clone());
        assert_eq!(root, storage.read_root());
    }

    #[test]
    #[should_panic(expected = "index out of bounds: the len is 1 but the index is 1")]
    fn no_change_to_cloned_index() {
        let storage = LocalStorage::default();
        let cloned_storage = storage.clone();
        let (_storage, next_id) = storage.push(SlotBase::new());
        let _ = cloned_storage[next_id];
    }

    #[test]
    fn append() {
        let storage = LocalStorage::default();
        let cloned_storage = storage.clone();
        let (storage, base_id) = storage.push(SlotBase::new());
        assert_eq!(base_id, SlotBaseId(1));
        assert_eq!(SlotBase::new(), storage[SlotBaseId(1)]);
        assert_eq!(cloned_storage.max_id() + 1, storage.max_id())
    }

    #[test]
    fn start_conditions() {
        let storage = LocalStorage::default();
        assert_eq!(SlotBaseId::ZERO, storage.max_id());
        assert_eq!(MapBase::empty(), storage.read_root());
        assert_eq!(SlotBase::new(), storage[SlotBaseId::ZERO]);
    }
}
