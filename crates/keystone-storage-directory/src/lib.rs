//! One bounded checkpoint register per private local Unix directory.
//!
//! Explicit creation never adopts an existing directory; recovery never creates
//! missing state. An OS lock lives for the store's lifetime. Publication stages,
//! synchronizes, renames, and synchronizes the directory before acknowledgment.
//! Trusted ancestry and a cooperative same-UID host are required. This is not an
//! authenticated store, rollback witness, network-filesystem or power-loss proof.

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;
