use std::time::Duration;

/// Whether an observation was accumulated exactly or after capacity was lost.
///
/// Saturation is sticky. Once an accumulator returns [`Self::SaturatedNow`],
/// every later offer returns [`Self::AlreadySaturated`] and leaves its public
/// value at the type's maximum.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "inspect whether the observation remained exact"]
pub enum Accumulation {
    /// The complete offered value was represented without arithmetic overflow.
    Exact,
    /// This offer overflowed and clamped the accumulator to its maximum.
    SaturatedNow,
    /// An earlier offer overflowed; this offer cannot restore exact knowledge.
    AlreadySaturated,
}

/// A non-authoritative `u64` observation counter with sticky overflow evidence.
///
/// Reaching `u64::MAX` exactly is still exact. A later positive addition is the
/// event that saturates the counter. After saturation, the value remains
/// `u64::MAX` even when zero is offered; use [`Self::is_saturated`] to avoid
/// mistaking the clamped value for an exact total.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SaturatingCounter {
    value: u64,
    saturated: bool,
}

impl SaturatingCounter {
    pub const fn new() -> Self {
        Self {
            value: 0,
            saturated: false,
        }
    }

    /// Constructs an exact observation, including an exact `u64::MAX` value.
    pub const fn from_value(value: u64) -> Self {
        Self {
            value,
            saturated: false,
        }
    }

    pub const fn value(self) -> u64 {
        self.value
    }

    pub const fn is_saturated(self) -> bool {
        self.saturated
    }

    pub fn increment(&mut self) -> Accumulation {
        self.add(1)
    }

    pub fn add(&mut self, amount: u64) -> Accumulation {
        if self.saturated {
            return Accumulation::AlreadySaturated;
        }

        match self.value.checked_add(amount) {
            Some(value) => {
                self.value = value;
                Accumulation::Exact
            }
            None => {
                self.value = u64::MAX;
                self.saturated = true;
                Accumulation::SaturatedNow
            }
        }
    }
}

/// A non-authoritative duration total with sticky overflow evidence.
///
/// Reaching [`Duration::MAX`] exactly is still exact. A later positive
/// addition is the event that saturates the accumulator. After saturation, the
/// value remains [`Duration::MAX`] for every later offer.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DurationAccumulator {
    value: Duration,
    saturated: bool,
}

impl DurationAccumulator {
    pub const fn new() -> Self {
        Self {
            value: Duration::ZERO,
            saturated: false,
        }
    }

    /// Constructs an exact observation, including an exact `Duration::MAX`.
    pub const fn from_value(value: Duration) -> Self {
        Self {
            value,
            saturated: false,
        }
    }

    pub const fn value(self) -> Duration {
        self.value
    }

    pub const fn is_saturated(self) -> bool {
        self.saturated
    }

    pub fn add(&mut self, amount: Duration) -> Accumulation {
        if self.saturated {
            return Accumulation::AlreadySaturated;
        }

        match self.value.checked_add(amount) {
            Some(value) => {
                self.value = value;
                Accumulation::Exact
            }
            None => {
                self.value = Duration::MAX;
                self.saturated = true;
                Accumulation::SaturatedNow
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_identity_and_exact_addition_preserve_exact_knowledge() {
        let mut counter = SaturatingCounter::new();
        assert_eq!(counter.add(0), Accumulation::Exact);
        assert_eq!(counter.add(41), Accumulation::Exact);
        assert_eq!(counter.increment(), Accumulation::Exact);
        assert_eq!(counter.value(), 42);
        assert!(!counter.is_saturated());
    }

    #[test]
    fn counter_maximum_can_be_exact_until_a_positive_offer_overflows() {
        let mut counter = SaturatingCounter::from_value(u64::MAX - 1);
        assert_eq!(counter.increment(), Accumulation::Exact);
        assert_eq!(counter.value(), u64::MAX);
        assert!(!counter.is_saturated());
        assert_eq!(counter.add(0), Accumulation::Exact);
        assert_eq!(counter.increment(), Accumulation::SaturatedNow);
        assert_eq!(counter.value(), u64::MAX);
        assert!(counter.is_saturated());
    }

    #[test]
    fn counter_saturation_is_sticky_and_value_is_maximum_after_overflow() {
        let mut counter = SaturatingCounter::from_value(u64::MAX);
        assert_eq!(counter.add(2), Accumulation::SaturatedNow);
        for amount in [0, 1, u64::MAX] {
            assert_eq!(counter.add(amount), Accumulation::AlreadySaturated);
            assert_eq!(counter.value(), u64::MAX);
            assert!(counter.is_saturated());
        }
    }

    #[test]
    fn duration_identity_and_exact_addition_preserve_exact_knowledge() {
        let mut total = DurationAccumulator::new();
        assert_eq!(total.add(Duration::ZERO), Accumulation::Exact);
        assert_eq!(total.add(Duration::from_millis(25)), Accumulation::Exact);
        assert_eq!(total.add(Duration::from_millis(17)), Accumulation::Exact);
        assert_eq!(total.value(), Duration::from_millis(42));
        assert!(!total.is_saturated());
    }

    #[test]
    fn duration_maximum_can_be_exact_until_a_positive_offer_overflows() {
        let one_nanosecond = Duration::from_nanos(1);
        let mut total = DurationAccumulator::from_value(Duration::MAX - one_nanosecond);
        assert_eq!(total.add(one_nanosecond), Accumulation::Exact);
        assert_eq!(total.value(), Duration::MAX);
        assert!(!total.is_saturated());
        assert_eq!(total.add(Duration::ZERO), Accumulation::Exact);
        assert_eq!(total.add(one_nanosecond), Accumulation::SaturatedNow);
        assert_eq!(total.value(), Duration::MAX);
        assert!(total.is_saturated());
    }

    #[test]
    fn duration_saturation_is_sticky_and_value_is_maximum_after_overflow() {
        let mut total = DurationAccumulator::from_value(Duration::MAX);
        assert_eq!(
            total.add(Duration::from_nanos(1)),
            Accumulation::SaturatedNow
        );
        for amount in [Duration::ZERO, Duration::from_secs(1), Duration::MAX] {
            assert_eq!(total.add(amount), Accumulation::AlreadySaturated);
            assert_eq!(total.value(), Duration::MAX);
            assert!(total.is_saturated());
        }
    }

    #[test]
    fn default_states_are_exact_additive_identities() {
        assert_eq!(SaturatingCounter::default(), SaturatingCounter::new());
        assert_eq!(DurationAccumulator::default(), DurationAccumulator::new());
    }
}
