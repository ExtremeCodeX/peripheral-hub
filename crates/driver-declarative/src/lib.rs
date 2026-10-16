//! Validated declarative drivers: TOML manifests interpreted into SDK factories.
//!
//! Community manifests are untrusted input. This crate parses them into typed
//! structures, enforces schema/semantic limits, and interprets only operations
//! it models. A manifest can never execute code, name a shell command, perform a
//! firmware/persistent-memory operation, or reach the transport budget directly:
//! the interpreter writes at most one bounded output/feature report per typed
//! `SetLighting` command and reports `NotSupported` for anything else.

mod driver;
mod error;
mod manifest;

pub use driver::*;
pub use error::*;
pub use manifest::*;

#[cfg(test)]
mod tests;
