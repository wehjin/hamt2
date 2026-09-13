use crate::trie::types::hash_key::HashKey;
use std::ops::Index;

pub struct HashKeyPath<const N: usize> {
    trie_keys: [HashKey; N],
}

impl<const N: usize> From<[i32; N]> for HashKeyPath<N> {
    fn from(key: [i32; N]) -> Self {
        let mut trie_keys = [HashKey::INVALID; N];
        for i in 0..N {
            trie_keys[i] = HashKey::new(key[i]);
        }
        Self { trie_keys }
    }
}

impl<const N: usize> Index<usize> for HashKeyPath<N> {
    type Output = HashKey;

    fn index(&self, index: usize) -> &Self::Output {
        &self.trie_keys[index]
    }
}
