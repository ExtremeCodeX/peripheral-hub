//! Typed, bounded macros and their validated local persistence.
//!
//! A macro is user configuration only: a named sequence of symbolic key and
//! delay actions with a finite repetition count. It never stores captured
//! keystrokes, raw reports, or credentials, and it is never executed here.
//! Persistence is delegated to `peripheral-hub-storage`; import/export uses an
//! explicit schema version. Storage, profile/button association and execution
//! are deliberately separate concerns: this crate only models and stores the
//! macro, so loading or importing a macro can never trigger an action.

mod error;
mod migrations;
mod model;
mod repository;

pub use error::*;
pub use migrations::*;
pub use model::*;
pub use repository::*;

#[cfg(test)]
mod tests;
