//! Identity-guarded ownership of one private Unix listener path.
//!
//! This physical adapter refuses an existing path, binds one listener, sets
//! mode `0600`, records its device/inode identity and removes the path on drop
//! only if that identity still names a socket. It does not select a directory,
//! authenticate peers, configure deadlines, frame traffic or run a daemon.

#[cfg(unix)]
mod unix;

#[cfg(unix)]
pub use unix::OwnedUnixListener;
