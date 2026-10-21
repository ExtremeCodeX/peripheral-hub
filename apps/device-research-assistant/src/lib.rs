//! Standalone Device Research Assistant.
//!
//! The DRA is a local development tool. It imports authorized textual captures,
//! correlates bounded payload differences, exports sanitized reports and emits
//! an explicitly non-loadable driver scaffold. It does not execute vendor
//! binaries, send or replay packets, or access devices beyond non-destructive
//! HID inventory when the user runs `devices list`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod analysis;
pub mod capture;
pub mod cli;
pub mod error;
pub mod inventory;
pub mod model;
pub mod paths;
pub mod report;
pub mod scaffold;
pub mod session;
