use crate::trie::BaseCommit;
use crate::trie::BaseView;

/// A trait for reading and writing Bases from storage.
#[allow(async_fn_in_trait)]
pub trait BaseEdit: BaseCommit + BaseView {}
