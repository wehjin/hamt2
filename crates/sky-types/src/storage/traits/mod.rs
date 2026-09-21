mod read;
mod write;

pub use read::*;
pub use write::*;

pub trait Storage<S> {
    fn storage(&self) -> &S;
}
