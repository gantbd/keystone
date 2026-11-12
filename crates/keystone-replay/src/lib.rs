//! Bounded exact-input replay retention with no eviction or retry policy.
//!
//! The register owns only a finite `(key, input, output)` relation. It does not
//! authorize, execute, retry, time, persist, or interpret any operation. A
//! caller must serialize [`ReplayRegister::classify`] and
//! [`ReplayRegister::record_new`] around its own interpretation. Construction
//! preallocates the complete entry vector fallibly, so recording a previously
//! classified new entry performs no further allocation.

use std::fmt;

/// The exhaustive result of offering one key and exact input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayDecision<T> {
    New,
    MatchingReplay(T),
    Conflict,
    Full,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayBuildError {
    Allocation,
}

/// A caller violated the serialized classify-then-record protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NotNew;

#[derive(Clone, Eq, PartialEq)]
struct Entry<K, I, O> {
    key: K,
    input: I,
    output: O,
}

/// Finite volatile replay relation. Entries remain until explicitly removed by
/// the owning domain lifecycle; capacity never evicts or replaces history.
#[derive(Clone, Eq, PartialEq)]
pub struct ReplayRegister<K, I, O> {
    entries: Vec<Entry<K, I, O>>,
    capacity: usize,
}

impl<K, I, O> fmt::Debug for ReplayRegister<K, I, O> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReplayRegister")
            .field("len", &self.entries.len())
            .field("capacity", &self.capacity)
            .finish_non_exhaustive()
    }
}

impl<K: Eq, I: Eq, O> ReplayRegister<K, I, O> {
    pub fn try_with_capacity(capacity: usize) -> Result<Self, ReplayBuildError> {
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(capacity)
            .map_err(|_| ReplayBuildError::Allocation)?;
        Ok(Self { entries, capacity })
    }

    /// Existing-key comparison precedes the capacity check. Consequently exact
    /// replay and conflict remain observable when the register is full.
    pub fn classify(&self, key: &K, input: &I) -> ReplayDecision<&O> {
        if let Some(entry) = self.entries.iter().find(|entry| &entry.key == key) {
            return if &entry.input == input {
                ReplayDecision::MatchingReplay(&entry.output)
            } else {
                ReplayDecision::Conflict
            };
        }
        if self.entries.len() >= self.capacity {
            ReplayDecision::Full
        } else {
            ReplayDecision::New
        }
    }

    /// Records an entry previously classified as [`ReplayDecision::New`].
    ///
    /// This rechecks the law at the mutation boundary. `NotNew` indicates caller
    /// misuse or intervening register mutation; it never replaces an entry.
    pub fn record_new(&mut self, key: K, input: I, output: O) -> Result<(), NotNew> {
        if !matches!(self.classify(&key, &input), ReplayDecision::New) {
            return Err(NotNew);
        }
        self.entries.push(Entry { key, input, output });
        Ok(())
    }

    /// Explicit owner-directed lifecycle removal, not overload eviction.
    pub fn remove_where(&mut self, mut remove: impl FnMut(&K, &I, &O) -> bool) -> usize {
        let before = self.entries.len();
        self.entries
            .retain(|entry| !remove(&entry.key, &entry.input, &entry.output));
        before - self.entries.len()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub const fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_four_outcomes_are_exact_and_lookup_precedes_full() {
        let mut register = ReplayRegister::try_with_capacity(1).unwrap();
        assert_eq!(register.classify(&1, &"a"), ReplayDecision::New);
        register.record_new(1, "a", 11).unwrap();
        assert_eq!(
            register.classify(&1, &"a"),
            ReplayDecision::MatchingReplay(&11)
        );
        assert_eq!(register.classify(&1, &"b"), ReplayDecision::Conflict);
        assert_eq!(register.classify(&2, &"a"), ReplayDecision::Full);
        assert_eq!(register.len(), 1);
    }

    #[test]
    fn zero_capacity_and_impossible_preallocation_are_closed() {
        let register = ReplayRegister::<u8, u8, u8>::try_with_capacity(0).unwrap();
        assert_eq!(register.classify(&1, &1), ReplayDecision::Full);
        assert_eq!(
            ReplayRegister::<u8, u8, u8>::try_with_capacity(usize::MAX),
            Err(ReplayBuildError::Allocation)
        );
    }

    #[test]
    fn conflicting_record_never_replaces_or_evicts() {
        let mut register = ReplayRegister::try_with_capacity(1).unwrap();
        register.record_new(1, "original", 7).unwrap();
        assert_eq!(register.record_new(1, "changed", 9), Err(NotNew));
        assert_eq!(
            register.classify(&1, &"original"),
            ReplayDecision::MatchingReplay(&7)
        );
    }

    #[test]
    fn explicit_lifecycle_removal_is_scoped_and_reopens_capacity() {
        let mut register = ReplayRegister::try_with_capacity(2).unwrap();
        register.record_new((1, 1), "a", 1).unwrap();
        register.record_new((2, 1), "b", 2).unwrap();
        assert_eq!(register.remove_where(|(owner, _), _, _| *owner == 1), 1);
        assert_eq!(register.classify(&(1, 2), &"c"), ReplayDecision::New);
        assert_eq!(
            register.classify(&(2, 1), &"b"),
            ReplayDecision::MatchingReplay(&2)
        );
    }

    #[test]
    fn debug_is_content_safe() {
        let mut register = ReplayRegister::try_with_capacity(1).unwrap();
        register
            .record_new("secret-key", "secret-input", "secret-output")
            .unwrap();
        let debug = format!("{register:?}");
        assert_eq!(debug, "ReplayRegister { len: 1, capacity: 1, .. }");
        assert!(!debug.contains("secret"));
    }

    #[test]
    fn exhaustive_small_sequences_match_the_mathematical_relation_model() {
        for capacity in 0..=3 {
            for encoded in 0_u32..6_u32.pow(5) {
                let mut register = ReplayRegister::try_with_capacity(capacity).unwrap();
                let mut model = Vec::<(u8, u8, u16)>::new();
                let mut sequence = encoded;
                for step in 0..5_u16 {
                    let offer = sequence % 6;
                    sequence /= 6;
                    let key = (offer / 2) as u8;
                    let input = (offer % 2) as u8;
                    let expected = match model.iter().find(|entry| entry.0 == key) {
                        Some(entry) if entry.1 == input => ReplayDecision::MatchingReplay(&entry.2),
                        Some(_) => ReplayDecision::Conflict,
                        None if model.len() >= capacity => ReplayDecision::Full,
                        None => ReplayDecision::New,
                    };
                    assert_eq!(register.classify(&key, &input), expected);
                    if expected == ReplayDecision::New {
                        let output = u16::from(key) * 100 + u16::from(input) * 10 + step;
                        register.record_new(key, input, output).unwrap();
                        model.push((key, input, output));
                    }
                    assert_eq!(register.len(), model.len());
                }
            }
        }
    }
}
