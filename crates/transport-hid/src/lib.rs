//! Non-destructive HID discovery and bounded report I/O using hidapi.

mod discovery;
mod io;
pub mod simulated;

pub use discovery::*;
pub use io::*;
