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
    pub fn read_root(&self) -> &MapBase {
        &self.head.root
    }
}

#[cfg(test)]
mod tests {
    use crate::local::storage::LocalStorage;
    use crate::types::slot_base::SlotBase;
    use sky_types::trie::{MapBase, SlotBaseId};

    #[test]
    fn start_conditions() {
        let storage = LocalStorage::default();
        assert_eq!(SlotBaseId::ZERO, storage.max_id());
        assert_eq!(&MapBase::empty(), storage.read_root());
        assert_eq!(SlotBase::new(), storage[SlotBaseId::ZERO]);
    }
}
