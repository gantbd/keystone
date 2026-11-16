//! Mechanism-level storage protocols.
//!
//! Domain engines should wrap these protocols with semantic record validation.
//! A committed result alone is not durability: commit and load evidence both
//! declare their acknowledgment strength. Neither a revision nor a mechanism
//! label authenticates a caller or establishes domain recovery correctness.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Revision(pub u64);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg(feature = "incubating-append-blob")]
pub struct Sequence(pub u64);

#[derive(Clone, Eq, PartialEq)]
pub struct Checkpoint {
    pub revision: Revision,
    pub bytes: Vec<u8>,
    pub strength: AcknowledgementStrength,
}

impl std::fmt::Debug for Checkpoint {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Checkpoint")
            .field("revision", &self.revision)
            .field("byte_length", &self.bytes.len())
            .finish()
    }
}

/// Evidence at the selected adapter boundary, not physical power-loss proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcknowledgementStrength {
    Volatile,
    Durable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitReceipt {
    pub revision: Revision,
    pub mechanism: &'static str,
    pub strength: AcknowledgementStrength,
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

/// Pre-offer refusal from a durable publication gate.
///
/// Neither case has consulted storage, so neither case changes the gate or
/// establishes uncertainty about the last confirmed state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitGateRefusal {
    Fenced,
    RevisionExhausted,
}

/// Failure after a candidate has been offered to storage.
///
/// Every variant is absorbing for the live gate: the caller can no longer
/// justify publishing either its old state or its candidate state. Recovery is
/// a separate consumer-owned operation over freshly loaded domain state.
#[derive(Debug, Eq, PartialEq)]
pub enum CommitGateFailure<E> {
    Store(E),
    Refused(StoreRefusal),
    Indeterminate,
    InvalidReceipt,
}

/// Mechanical publish-after-durable-receipt gate for one serialized owner.
///
/// The gate owns no checkpoint schema, candidate state, outcome, recovery
/// policy, retry policy or domain error. A consumer prepares and encodes its
/// candidate before calling storage, asks this gate for the exact next
/// revision, then resolves the one storage outcome. Only an exact durable
/// receipt advances the confirmed revision. Every other post-offer result
/// permanently fences this instance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommitGate {
    confirmed: Option<Revision>,
    fenced: bool,
}

impl CommitGate {
    pub const fn new(confirmed: Option<Revision>) -> Self {
        Self {
            confirmed,
            fenced: false,
        }
    }

    pub const fn confirmed_revision(self) -> Option<Revision> {
        self.confirmed
    }

    pub const fn is_fenced(self) -> bool {
        self.fenced
    }

    /// Returns the only revision that may be offered next.
    ///
    /// Exhaustion and an already-fenced instance refuse before storage is
    /// called and therefore do not alter the gate.
    pub fn next_revision(self) -> Result<Revision, CommitGateRefusal> {
        if self.fenced {
            return Err(CommitGateRefusal::Fenced);
        }
        match self.confirmed {
            None => Ok(Revision(0)),
            Some(Revision(value)) => value
                .checked_add(1)
                .map(Revision)
                .ok_or(CommitGateRefusal::RevisionExhausted),
        }
    }

    /// Resolves one completed storage offer for `proposed`.
    ///
    /// Callers must publish their prepared domain candidate and result only
    /// after this method succeeds.
    pub fn resolve<E>(
        &mut self,
        proposed: Revision,
        outcome: Result<CommitOutcome, E>,
    ) -> Result<(), CommitGateFailure<E>> {
        let expected = self.next_revision();
        let result = match (expected, outcome) {
            (Ok(expected), Ok(CommitOutcome::Committed(receipt)))
                if proposed == expected
                    && receipt.revision == proposed
                    && receipt.strength == AcknowledgementStrength::Durable =>
            {
                Ok(())
            }
            (Ok(_), Ok(CommitOutcome::Committed(_))) => Err(CommitGateFailure::InvalidReceipt),
            (Ok(_), Ok(CommitOutcome::Refused(refusal))) => {
                Err(CommitGateFailure::Refused(refusal))
            }
            (Ok(_), Ok(CommitOutcome::Indeterminate)) => Err(CommitGateFailure::Indeterminate),
            (Ok(_), Err(error)) => Err(CommitGateFailure::Store(error)),
            // Supplying an already-produced storage outcome to a fenced or
            // exhausted gate is a caller protocol violation. The offered
            // result cannot restore a justified live state.
            (Err(_), _) => Err(CommitGateFailure::InvalidReceipt),
        };
        match result {
            Ok(()) => {
                self.confirmed = Some(proposed);
                Ok(())
            }
            Err(error) => {
                self.fenced = true;
                Err(error)
            }
        }
    }
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
#[cfg(feature = "incubating-append-blob")]
pub struct AppendReceipt {
    pub sequence: Sequence,
    pub mechanism: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg(feature = "incubating-append-blob")]
pub enum AppendOutcome {
    Appended(AppendReceipt),
    Refused(StoreRefusal),
    Indeterminate,
}

/// Ordered append protocol. An implementation is not a checkpoint store adapter.
#[cfg(feature = "incubating-append-blob")]
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
#[cfg(feature = "incubating-append-blob")]
pub enum BlobPutOutcome {
    Created,
    AlreadyPresent,
    Conflict,
    Refused(StoreRefusal),
    Indeterminate,
}

/// Immutable publication protocol. Content identity is supplied by the domain.
#[cfg(feature = "incubating-append-blob")]
pub trait BlobStore {
    type Error;

    fn put_if_absent(&mut self, key: &str, bytes: &[u8]) -> Result<BlobPutOutcome, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn receipt(revision: Revision, strength: AcknowledgementStrength) -> CommitOutcome {
        CommitOutcome::Committed(CommitReceipt {
            revision,
            mechanism: "test",
            strength,
        })
    }

    #[test]
    fn exact_durable_receipts_alone_advance_confirmed_revision() {
        let mut gate = CommitGate::new(None);
        let first = gate.next_revision().unwrap();
        assert_eq!(first, Revision(0));
        gate.resolve::<()>(first, Ok(receipt(first, AcknowledgementStrength::Durable)))
            .unwrap();
        assert_eq!(gate.confirmed_revision(), Some(first));
        assert_eq!(gate.next_revision(), Ok(Revision(1)));
        assert!(!gate.is_fenced());
    }

    #[test]
    fn every_post_offer_failure_is_absorbing() {
        let failures: [Result<CommitOutcome, &'static str>; 5] = [
            Ok(receipt(Revision(0), AcknowledgementStrength::Volatile)),
            Ok(receipt(Revision(1), AcknowledgementStrength::Durable)),
            Ok(CommitOutcome::Refused(StoreRefusal::ValueLimit)),
            Ok(CommitOutcome::Indeterminate),
            Err("private store error"),
        ];
        for failure in failures {
            let mut gate = CommitGate::new(None);
            assert!(gate.resolve(Revision(0), failure).is_err());
            assert!(gate.is_fenced());
            assert_eq!(gate.confirmed_revision(), None);
            assert_eq!(gate.next_revision(), Err(CommitGateRefusal::Fenced));
        }
    }

    #[test]
    fn revision_exhaustion_refuses_before_offer_without_fencing() {
        let gate = CommitGate::new(Some(Revision(u64::MAX)));
        assert_eq!(
            gate.next_revision(),
            Err(CommitGateRefusal::RevisionExhausted)
        );
        assert!(!gate.is_fenced());
        assert_eq!(gate.confirmed_revision(), Some(Revision(u64::MAX)));
    }
}
