use crate::local::storage::LocalStorage;
use sky_types::trie::MapBase;

pub struct LocalTrie<'a> {
    storage: LocalStorage<'a>,
}
impl<'a> Default for LocalTrie<'a> {
    fn default() -> Self {
        let storage = LocalStorage::default();
        Self { storage }
    }
}

impl<'a> LocalTrie<'a> {
    pub fn root(&self) -> MapBase {
        self.storage.read_root()
    }
}

#[cfg(test)]
mod tests {
    use crate::local::trie::LocalTrie;
    use sky_types::trie::MapBase;

    #[test]
    fn start_conditions() {
        let local = LocalTrie::default();
        assert_eq!(MapBase::empty(), local.root());
    }
}
