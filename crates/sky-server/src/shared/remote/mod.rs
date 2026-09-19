mod client;
mod readonly;
pub use client::*;
pub use readonly::*;

/// Users of remote storages must provide an implementation of this trait.
pub trait SpawnTask: Clone + Send + Sync + 'static {
    fn spawn_task(future: impl Future<Output = ()> + 'static);
}

#[cfg(test)]
mod tests;
