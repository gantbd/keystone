//! Truthful process-local volatile checkpoint storage.

use std::collections::BTreeMap;
use std::convert::Infallible;

use keystone_storage::{
    AcknowledgementStrength, Checkpoint, CheckpointStore, CommitOutcome, CommitReceipt, Revision,
    StoreRefusal,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VolatileLimits {
    pub max_entries: usize,
    pub max_key_bytes: usize,
    pub max_value_bytes: usize,
    pub max_total_bytes: usize,
}

pub struct VolatileCheckpointStore {
    limits: VolatileLimits,
    entries: BTreeMap<String, Checkpoint>,
    retained_bytes: usize,
}

impl std::fmt::Debug for VolatileCheckpointStore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VolatileCheckpointStore")
            .field("limits", &self.limits)
            .field("entries", &self.entries.len())
            .field("retained_bytes", &self.retained_bytes)
            .finish()
    }
}

impl VolatileCheckpointStore {
    pub fn new(limits: VolatileLimits) -> Self {
        Self {
            limits,
            entries: BTreeMap::new(),
            retained_bytes: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
}

impl CheckpointStore for VolatileCheckpointStore {
    type Error = Infallible;

    fn load(&self, key: &str) -> Result<Option<Checkpoint>, Self::Error> {
        Ok(self.entries.get(key).cloned())
    }

    fn compare_exchange(
        &mut self,
        key: &str,
        expected: Option<Revision>,
        bytes: &[u8],
    ) -> Result<CommitOutcome, Self::Error> {
        if key.is_empty() {
            return Ok(CommitOutcome::Refused(StoreRefusal::InvalidKey));
        }
        if key.len() > self.limits.max_key_bytes {
            return Ok(CommitOutcome::Refused(StoreRefusal::KeyLimit));
        }
        if bytes.len() > self.limits.max_value_bytes {
            return Ok(CommitOutcome::Refused(StoreRefusal::ValueLimit));
        }

        let current = self.entries.get(key);
        let actual = current.map(|entry| entry.revision);
        if actual != expected {
            return Ok(CommitOutcome::Refused(StoreRefusal::RevisionConflict {
                expected,
                actual,
            }));
        }
        if current.is_none() && self.entries.len() >= self.limits.max_entries {
            return Ok(CommitOutcome::Refused(StoreRefusal::EntryLimit));
        }

        let revision = match actual {
            Some(current) => match current.0.checked_add(1) {
                Some(next) => Revision(next),
                None => return Ok(CommitOutcome::Refused(StoreRefusal::RevisionExhausted)),
            },
            None => Revision(0),
        };
        let old_bytes = current.map_or(0, |entry| entry.bytes.len());
        let Some(base) = self.retained_bytes.checked_sub(old_bytes) else {
            return Ok(CommitOutcome::Indeterminate);
        };
        let Some(next_retained_bytes) = base.checked_add(bytes.len()) else {
            return Ok(CommitOutcome::Refused(StoreRefusal::TotalBytesLimit));
        };
        if next_retained_bytes > self.limits.max_total_bytes {
            return Ok(CommitOutcome::Refused(StoreRefusal::TotalBytesLimit));
        }

        self.entries.insert(
            key.to_owned(),
            Checkpoint {
                revision,
                bytes: bytes.to_vec(),
                strength: AcknowledgementStrength::Volatile,
            },
        );
        self.retained_bytes = next_retained_bytes;
        Ok(CommitOutcome::Committed(CommitReceipt {
            revision,
            mechanism: "volatile-process-memory-v1",
            strength: AcknowledgementStrength::Volatile,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> VolatileCheckpointStore {
        VolatileCheckpointStore::new(VolatileLimits {
            max_entries: 1,
            max_key_bytes: 4,
            max_value_bytes: 4,
            max_total_bytes: 4,
        })
    }

    #[test]
    fn exact_predecessor_controls_publication() {
        let mut store = store();
        assert!(matches!(
            store.compare_exchange("key", None, b"one"),
            Ok(CommitOutcome::Committed(CommitReceipt {
                revision: Revision(0),
                ..
            }))
        ));
        let before = store.load("key").unwrap();
        assert_eq!(
            store.compare_exchange("key", None, b"two").unwrap(),
            CommitOutcome::Refused(StoreRefusal::RevisionConflict {
                expected: None,
                actual: Some(Revision(0)),
            })
        );
        assert_eq!(store.load("key").unwrap(), before);
    }

    #[test]
    fn capacity_refusal_preserves_existing_record_and_account() {
        let mut store = store();
        store.compare_exchange("key", None, b"data").unwrap();
        let before = store.load("key").unwrap();
        assert_eq!(
            store
                .compare_exchange("key", Some(Revision(0)), b"large")
                .unwrap(),
            CommitOutcome::Refused(StoreRefusal::ValueLimit)
        );
        assert_eq!(store.load("key").unwrap(), before);
        assert_eq!(store.retained_bytes(), 4);
    }

    #[test]
    fn volatile_receipts_and_loads_never_claim_durability_and_debug_redacts_content() {
        let mut store = store();
        let CommitOutcome::Committed(receipt) =
            store.compare_exchange("key", None, b"data").unwrap()
        else {
            panic!("commit expected");
        };
        assert_eq!(receipt.strength, AcknowledgementStrength::Volatile);
        let checkpoint = store.load("key").unwrap().unwrap();
        assert_eq!(checkpoint.strength, AcknowledgementStrength::Volatile);
        assert!(!format!("{checkpoint:?}").contains("data"));
        let debug = format!("{store:?}");
        assert!(!debug.contains("\"key\""));
        assert!(!debug.contains("data"));
    }
}
