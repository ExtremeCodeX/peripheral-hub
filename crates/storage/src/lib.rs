//! Embedded SQLite persistence with explicit, ordered, versioned migrations.
//!
//! This crate is domain-agnostic: it owns the connection policy and migration
//! bookkeeping only. Feature crates supply their own [`Migration`] sets and run
//! them through [`Database::migrate`]. No user data, vendor binary, or raw
//! capture is ever written here by convention; callers store only validated,
//! structured state (for example, sanitized profiles).

mod database;
mod migration;

pub use database::*;
pub use migration::*;

/// Re-export the pinned SQLite driver so feature crates depend on one version.
pub use rusqlite;
