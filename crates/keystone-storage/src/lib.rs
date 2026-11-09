//! Mechanism-level storage protocols.
//!
//! Domain engines should wrap these protocols with semantic record validation.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Revision(pub u64);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Sequence(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Checkpoint {
    pub revision: Revision,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitReceipt {
    pub revision: Revision,
    pub mechanism: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StoreRefusal {
    InvalidKey,
    KeyLimit,
    ValueLimit,
    EntryLimit,
    TotalBytesLimit,
    RevisionExhausted,
    RevisionConflict {
        expected: Option<Revision>,
        actual: Option<Revision>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommitOutcome {
    Committed(CommitReceipt),
    Refused(StoreRefusal),
    Indeterminate,
}

/// Revisioned replacement protocol. This is intentionally distinct from append.
pub trait CheckpointStore {
    type Error;

    fn load(&self, key: &str) -> Result<Option<Checkpoint>, Self::Error>;

    fn compare_exchange(
        &mut self,
        key: &str,
        expected: Option<Revision>,
        bytes: &[u8],
    ) -> Result<CommitOutcome, Self::Error>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppendReceipt {
    pub sequence: Sequence,
    pub mechanism: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppendOutcome {
    Appended(AppendReceipt),
    Refused(StoreRefusal),
    Indeterminate,
}

/// Ordered append protocol. An implementation is not a checkpoint store adapter.
pub trait AppendStore {
    type Error;

    fn append(
        &mut self,
        stream: &str,
        expected_predecessor: Option<Sequence>,
        bytes: &[u8],
    ) -> Result<AppendOutcome, Self::Error>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlobPutOutcome {
    Created,
    AlreadyPresent,
    Conflict,
    Refused(StoreRefusal),
    Indeterminate,
}

/// Immutable publication protocol. Content identity is supplied by the domain.
pub trait BlobStore {
    type Error;

    fn put_if_absent(&mut self, key: &str, bytes: &[u8]) -> Result<BlobPutOutcome, Self::Error>;
}
