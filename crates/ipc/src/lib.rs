//! Typed, framed, same-user local IPC, independent of SDK contract versions.

mod framing;
mod local;
mod protocol;
pub use framing::*;
pub use local::*;
pub use protocol::*;
