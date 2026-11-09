//! Bounded, non-authoritative observation storage. Event meaning stays with
//! consumers; this crate reads no clock and invokes no subscribers or exporters.

mod capture;
pub use capture::{Capture, CaptureBuffer};

/// Compatibility append-only facade. `new(0)` still counts every offer as loss;
/// `events()` keeps its contiguous borrowed slice. For fallible construction,
/// explicit disabling and FIFO draining, use [`CaptureBuffer`] directly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Recorder<E> {
    buffer: CaptureBuffer<E>,
}

impl<E> Recorder<E> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: CaptureBuffer::try_with_capacity(capacity)
                .expect("recorder storage allocation failed"),
        }
    }

    pub fn record(&mut self, event: E) {
        self.buffer.record(event);
    }

    pub fn events(&self) -> &[E] {
        self.buffer.append_only_events()
    }

    pub fn dropped(&self) -> u64 {
        self.buffer.dropped()
    }

    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    pub fn is_saturated(&self) -> bool {
        self.buffer.is_saturated()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saturation_drops_observations_without_replacing_history() {
        let mut recorder = Recorder::new(1);
        recorder.record("first");
        recorder.record("second");
        assert_eq!(recorder.events(), &["first"]);
        assert_eq!(recorder.dropped(), 1);
    }

    #[test]
    fn zero_capacity_is_truthfully_saturated() {
        let mut recorder = Recorder::new(0);
        recorder.record("dropped");
        assert!(recorder.events().is_empty());
        assert!(recorder.is_saturated());
        assert_eq!(recorder.dropped(), 1);
    }

    #[test]
    fn append_only_slice_and_clone_remain_compatible() {
        let mut original = Recorder::new(3);
        original.record("first");
        let mut cloned = original.clone();
        cloned.record("second");
        cloned.record("third");
        cloned.record("lost");
        assert_eq!(original.events(), &["first"]);
        assert_eq!(cloned.events(), &["first", "second", "third"]);
        assert_eq!(original.capacity(), cloned.capacity());
        assert_eq!(original.dropped(), 0);
        assert_eq!(cloned.dropped(), 1);
        assert!(cloned.is_saturated());
        assert!(!original.is_saturated());
    }
}
