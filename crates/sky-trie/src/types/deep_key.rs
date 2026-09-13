use crate::types::HashKey;
use std::ops::Index;

pub struct DeepKey<const N: usize> {
    keys: [HashKey; N],
}

impl<const N: usize> From<[i32; N]> for DeepKey<N> {
    fn from(key: [i32; N]) -> Self {
        let mut keys = [HashKey::INVALID; N];
        for i in 0..N {
            keys[i] = HashKey::new(key[i]);
        }
        Self { keys }
    }
}

impl<const N: usize> Index<usize> for DeepKey<N> {
    type Output = HashKey;

    fn index(&self, index: usize) -> &Self::Output {
        &self.keys[index]
    }
}
