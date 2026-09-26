use crate::storage::mem::MemView;

/// Deliberately non-Clone
#[derive(Debug)]
pub struct MemLoad {
    pub(crate) _inner: MemView,
}
