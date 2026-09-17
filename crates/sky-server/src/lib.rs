//! Server layer for the sky-db stack.
//!
//! The `server` feature (on by default) gates the `server` module: the
//! storage service and the socket request processor. Disable it for
//! client-side consumers that only need the `shared` wire types.

pub mod shared;
#[cfg(feature = "server")]
pub mod server;
