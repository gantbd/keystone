use std::collections::{TryReserveError, VecDeque};
use std::fmt;

/// Storage disposition only, never evidence about a domain operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capture {
    Recorded,
    Dropped,
    Disabled,
}

/// Preallocated FIFO capture with drop-newest overload behavior.
///
/// Construct before domain execution. Recording and draining never grow the
/// reserved storage, format events, sample clocks, or call subscribers. An
/// event's own destructor may run when it is discarded; use bounded, inert
/// event values on latency-sensitive paths. This is a record-count bound, not
/// a bound on heap allocations owned by arbitrary `E` values.
#[derive(Eq, PartialEq)]
pub struct CaptureBuffer<E> {
    records: VecDeque<E>,
    limit: usize,
    dropped: u64,
    enabled: bool,
}

// VecDeque::clone need not retain spare capacity. Reserve the whole configured
// ceiling so a partially filled clone also preserves allocation-free recording.
impl<E: Clone> Clone for CaptureBuffer<E> {
    fn clone(&self) -> Self {
        let mut cloned = if self.enabled {
            Self::try_with_capacity(self.limit).expect("capture storage allocation failed")
        } else {
            Self::disabled()
        };
        cloned.records.extend(self.records.iter().cloned());
        cloned.dropped = self.dropped;
        cloned
    }
}

impl<E> fmt::Debug for CaptureBuffer<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CaptureBuffer")
            .field("queued", &self.queued())
            .field("capacity", &self.limit)
            .field("dropped", &self.dropped)
            .field("enabled", &self.enabled)
            .finish_non_exhaustive()
    }
}

impl<E> CaptureBuffer<E> {
    /// An enabled zero-capacity buffer drops every offer. Use `disabled()` to
    /// opt out without counting offers as overload loss.
    pub fn try_with_capacity(limit: usize) -> Result<Self, TryReserveError> {
        let mut records = VecDeque::new();
        records.try_reserve_exact(limit)?;
        Ok(Self {
            records,
            limit,
            dropped: 0,
            enabled: true,
        })
    }

    /// No allocation; offered values are discarded without incrementing loss.
    pub fn disabled() -> Self {
        Self {
            records: VecDeque::new(),
            limit: 0,
            dropped: 0,
            enabled: false,
        }
    }

    pub fn record(&mut self, event: E) -> Capture {
        if !self.enabled {
            return Capture::Disabled;
        }
        if self.records.len() == self.limit {
            self.dropped = self.dropped.saturating_add(1);
            return Capture::Dropped;
        }
        self.records.push_back(event);
        Capture::Recorded
    }

    pub fn pop_front(&mut self) -> Option<E> {
        self.records.pop_front()
    }

    pub fn queued(&self) -> usize {
        self.records.len()
    }

    /// Configured logical ceiling, independent of allocator rounding.
    pub fn capacity(&self) -> usize {
        self.limit
    }

    /// Explicit opt-out differs from an enabled zero-capacity recorder.
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Cumulative overload loss since construction, saturating at `u64::MAX`.
    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    pub fn is_saturated(&self) -> bool {
        self.queued() == self.limit
    }

    // The legacy append-only facade never drains, so its deque stays contiguous.
    pub(super) fn append_only_events(&self) -> &[E] {
        let (events, wrapped) = self.records.as_slices();
        debug_assert!(wrapped.is_empty());
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_wraparound_reuses_reserved_storage_and_never_replaces_retained_records() {
        let mut buffer = CaptureBuffer::try_with_capacity(3).unwrap();
        let reserved = buffer.records.capacity();
        for n in 0..3 {
            assert_eq!(buffer.record(n), Capture::Recorded);
        }
        assert_eq!(buffer.record(99), Capture::Dropped);
        for n in 0..100 {
            assert_eq!(buffer.pop_front(), Some(n));
            assert_eq!(buffer.record(n + 3), Capture::Recorded);
            assert_eq!(buffer.records.capacity(), reserved);
            assert_eq!(buffer.queued(), 3);
        }
        assert_eq!(buffer.pop_front(), Some(100));
        assert_eq!(buffer.pop_front(), Some(101));
        assert_eq!(buffer.pop_front(), Some(102));
        assert_eq!(buffer.pop_front(), None);
        assert_eq!(buffer.dropped(), 1);
    }

    #[test]
    fn disabled_and_enabled_zero_capacity_have_distinct_loss_meaning() {
        let mut disabled = CaptureBuffer::disabled();
        let mut full = CaptureBuffer::try_with_capacity(0).unwrap();
        assert!(!disabled.is_enabled());
        assert!(full.is_enabled());
        for _ in 0..3 {
            assert_eq!(disabled.record(1), Capture::Disabled);
            assert_eq!(full.record(1), Capture::Dropped);
        }
        assert_eq!(disabled.dropped(), 0);
        assert_eq!(full.dropped(), 3);
        assert_eq!(disabled.records.capacity(), 0);
        assert!(disabled.is_saturated());
        assert!(full.is_saturated());
        assert_eq!(disabled.pop_front(), None);
        assert_eq!(full.pop_front(), None);
    }

    #[test]
    fn loss_saturates_and_survives_drain() {
        let mut buffer = CaptureBuffer::try_with_capacity(1).unwrap();
        assert_eq!(buffer.record(1), Capture::Recorded);
        buffer.dropped = u64::MAX;
        assert_eq!(buffer.record(2), Capture::Dropped);
        assert_eq!(buffer.dropped(), u64::MAX);
        assert_eq!(buffer.pop_front(), Some(1));
        assert_eq!(buffer.record(3), Capture::Recorded);
        assert_eq!(buffer.dropped(), u64::MAX);
    }

    #[test]
    fn impossible_preallocation_is_a_fallible_construction_error() {
        assert!(CaptureBuffer::<u64>::try_with_capacity(usize::MAX).is_err());
    }

    #[test]
    fn partial_clones_retain_the_full_reservation_and_independent_fifo() {
        let mut original = CaptureBuffer::try_with_capacity(8).unwrap();
        original.record(1);
        original.record(2);
        assert_eq!(original.pop_front(), Some(1));
        let mut cloned = original.clone();
        let reserved = cloned.records.capacity();
        assert!(reserved >= 8);
        for n in 3..10 {
            assert_eq!(cloned.record(n), Capture::Recorded);
        }
        assert_eq!(cloned.records.capacity(), reserved);
        assert_eq!(cloned.record(10), Capture::Dropped);
        assert_eq!(original.queued(), 1);
        assert_eq!(original.dropped(), 0);
        assert_eq!(cloned.pop_front(), Some(2));
    }

    #[test]
    fn debug_does_not_require_or_disclose_event_debug() {
        struct Private;
        let mut buffer = CaptureBuffer::try_with_capacity(1).unwrap();
        buffer.record(Private);
        let debug = format!("{buffer:?}");
        assert!(debug.contains("queued: 1"));
        assert!(!debug.contains("Private"));
    }
}
