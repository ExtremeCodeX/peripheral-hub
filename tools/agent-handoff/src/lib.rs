//! Agent-facing gate between DRA evidence and driver work.
//!
//! The tool consumes a sanitized DRA `report.json`, validates it with the DRA's
//! typed model plus additional cross-document checks, records a SHA-256
//! provenance digest, and writes a contribution skeleton. It never generates
//! driver code, a loadable manifest, a capability claim or a hardware result.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod cli;
pub mod error;
pub mod handoff;
pub mod manifest;
pub mod model;
pub mod report_gate;
