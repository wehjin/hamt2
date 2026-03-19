use crate::hamt::trie::key::TrieKey;
use std::ops::Index;

pub struct DeepKey<const N: usize> {
    trie_keys: [TrieKey; N],
}

impl<const N: usize> From<[i32; N]> for DeepKey<N> {
    fn from(key: [i32; N]) -> Self {
        let mut trie_keys = [TrieKey::INVALID; N];
        for i in 0..N {
            trie_keys[i] = TrieKey::new(key[i]);
        }
        Self { trie_keys }
    }
}

impl<const N: usize> Index<usize> for DeepKey<N> {
    type Output = TrieKey;

    fn index(&self, index: usize) -> &Self::Output {
        &self.trie_keys[index]
    }
}
