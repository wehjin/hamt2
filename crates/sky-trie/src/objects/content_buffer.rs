use crate::{Base, Buffer, BufferIndex, BufferMut, MapBase, Slot};
use std::collections::HashMap;
use std::sync::Arc;

/// A content-addressed in-memory buffer.
///
/// Each base and byte array is stored under its BLAKE3 hash, which is the same
/// value iroh uses to store and fetch the corresponding blob. The `past`
/// pointer supports the persistent (snapshot) structure, mirroring the old
/// `VecBuffer`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ContentBuffer {
    past: Option<Arc<ContentBuffer>>,
    bases: HashMap<BufferIndex, Base>,
    bytes: HashMap<BufferIndex, Vec<u8>>,
    root: MapBase,
}

impl ContentBuffer {
    pub fn new() -> Self {
        Self {
            past: None,
            bases: HashMap::new(),
            bytes: HashMap::new(),
            root: MapBase::empty(),
        }
    }

    pub fn extend(past: Arc<Self>) -> Self {
        let root = past.root;
        Self {
            past: Some(past),
            bases: HashMap::new(),
            bytes: HashMap::new(),
            root,
        }
    }

    pub fn retract(self) -> Self {
        if let Some(past) = self.past {
            let mut bases = past.bases.clone();
            bases.extend(self.bases);
            let mut bytes = past.bytes.clone();
            bytes.extend(self.bytes);
            Self {
                past: None,
                bases,
                bytes,
                root: self.root,
            }
        } else {
            self
        }
    }
}

impl Buffer for ContentBuffer {
    fn get_root(&self) -> MapBase {
        self.root
    }

    fn get_base(&self, id: BufferIndex, _size: usize) -> &[Slot] {
        if let Some(base) = self.bases.get(&id) {
            return &base.slots[..];
        }
        if let Some(past) = &self.past {
            return past.get_base(id, _size);
        }
        &[]
    }

    fn get_bytes(&self, index: BufferIndex) -> Vec<u8> {
        if let Some(bytes) = self.bytes.get(&index) {
            return bytes.clone();
        }
        if let Some(past) = &self.past {
            return past.get_bytes(index);
        }
        vec![]
    }
}

impl BufferMut for ContentBuffer {
    async fn push_root(&mut self, root: MapBase) {
        self.root = root;
    }

    async fn push_base(&mut self, base: Base) -> BufferIndex {
        let bytes = postcard::to_allocvec(&base).expect("base should serialize");
        let index = BufferIndex(blake3::hash(&bytes));
        self.bases.insert(index, base);
        index
    }

    async fn push_bytes(&mut self, bytes: impl AsRef<[u8]>) -> BufferIndex {
        let bytes = bytes.as_ref().to_vec();
        let index = BufferIndex(blake3::hash(&bytes));
        self.bytes.insert(index, bytes);
        index
    }
}

#[cfg(test)]
mod tests {
    use crate::{Buffer, BufferMut, SkyTrieMut};

    #[tokio::test]
    async fn buffer_pushes_and_gets_bytes() {
        let mut trie = SkyTrieMut::new();
        let mut bytes = b"hello"[..].to_vec();
        for _ in 0..7 {
            let index = trie.push_bytes(&bytes).await;
            let get_bytes = trie.get_bytes(index);
            assert_eq!(get_bytes, bytes);
            bytes.extend(bytes.clone());
        }
    }
}
