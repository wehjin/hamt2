use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;

/// A handle into the trie's buffer.
///
/// This is a content address: the BLAKE3 hash of the serialized base or byte
/// array it refers to. It is the same value iroh uses to store and fetch the
/// corresponding blob, so the in-memory and persisted representations agree.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct BufferIndex(pub blake3::Hash);

impl Default for BufferIndex {
    fn default() -> Self {
        Self::NIL
    }
}

impl Display for BufferIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl BufferIndex {
    /// The empty-buffer sentinel. Never dereferenced; used as the `base` of an
    /// empty [`MapBase`](crate::MapBase).
    pub const NIL: BufferIndex = BufferIndex(blake3::Hash::from_bytes([0; 32]));
}
