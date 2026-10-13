//! Versioned, transport-independent device contracts.
//!
//! This crate performs validation only. It does not enumerate, open, or write
//! physical devices. See `docs/architecture/sdk.md` for compatibility rules.

mod capabilities;
mod driver;
mod error;
mod identity;
mod matching;
mod transport;
mod version;

pub use capabilities::*;
pub use driver::*;
pub use error::*;
pub use identity::*;
pub use matching::*;
pub use transport::*;
pub use version::*;

#[cfg(test)]
mod tests;
