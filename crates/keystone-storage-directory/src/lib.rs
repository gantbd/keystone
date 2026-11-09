//! Placeholder for a future qualified directory checkpoint adapter.
//!
//! No filesystem implementation or durability claim exists in this crate yet.

use keystone_storage::CheckpointStore;

pub const PROJECT_STAGE: &str = "protocol-blocked-scaffold";

/// Marker proving the scaffold is compiled against the selected protocol.
pub fn protocol_marker<S: CheckpointStore>(_store: &S) {}
