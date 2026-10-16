//! Versioned device profiles and their validated local persistence.
//!
//! A profile is user configuration only: a display name plus, optionally, a
//! bound device model and a typed lighting request. Profiles never carry raw
//! reports, vendor payloads, or credentials, and persistence is delegated to
//! `peripheral-hub-storage`. Import/export uses an explicit schema version.

mod error;
mod migrations;
mod profile;
mod repository;

pub use error::*;
pub use migrations::*;
pub use profile::*;
pub use repository::*;

#[cfg(test)]
mod tests;
