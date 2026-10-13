//! Pure device-inventory orchestration over SDK descriptors and the Registry.
//!
//! Callers supply complete snapshots. This is not discovery, hotplug monitoring,
//! transport access, daemon connection management, or profile persistence.

mod inventory;

pub use inventory::*;

#[cfg(test)]
mod tests;
