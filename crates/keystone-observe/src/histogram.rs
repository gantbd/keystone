use std::time::Duration;

use crate::{Accumulation, DurationAccumulator, SaturatingCounter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixedDurationHistogramBuildError {
    EmptyBounds,
    BoundsNotStrictlyIncreasing,
}

/// A bounded duration distribution with closed, strictly ordered upper bounds.
///
/// Values equal to a bound enter that bound's bucket. Values above the final
/// bound enter the overflow bucket. The histogram reads no clock and performs
/// no export; consumers choose the timing boundary and observation policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FixedDurationHistogram<const N: usize> {
    upper_bounds: [Duration; N],
    buckets: [SaturatingCounter; N],
    overflow: SaturatingCounter,
    count: SaturatingCounter,
    sum: DurationAccumulator,
    fidelity_lost: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FixedDurationHistogramSnapshot<const N: usize> {
    pub upper_bounds: [Duration; N],
    pub bucket_counts: [u64; N],
    pub overflow_count: u64,
    pub count: u64,
    pub sum: Duration,
    pub fidelity_lost: bool,
}

impl<const N: usize> FixedDurationHistogram<N> {
    pub fn try_new(upper_bounds: [Duration; N]) -> Result<Self, FixedDurationHistogramBuildError> {
        if N == 0 {
            return Err(FixedDurationHistogramBuildError::EmptyBounds);
        }
        if upper_bounds.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(FixedDurationHistogramBuildError::BoundsNotStrictlyIncreasing);
        }
        Ok(Self {
            upper_bounds,
            buckets: [SaturatingCounter::new(); N],
            overflow: SaturatingCounter::new(),
            count: SaturatingCounter::new(),
            sum: DurationAccumulator::new(),
            fidelity_lost: false,
        })
    }

    /// Records one caller-measured duration without affecting caller decisions.
    pub fn observe(&mut self, value: Duration) -> Accumulation {
        if self.fidelity_lost {
            return Accumulation::AlreadySaturated;
        }
        let count = self.count.increment();
        let bucket = match self
            .upper_bounds
            .iter()
            .position(|upper_bound| value <= *upper_bound)
        {
            Some(index) => self.buckets[index].increment(),
            None => self.overflow.increment(),
        };
        let sum = self.sum.add(value);
        if [count, bucket, sum]
            .into_iter()
            .any(|result| result == Accumulation::SaturatedNow)
        {
            self.fidelity_lost = true;
            Accumulation::SaturatedNow
        } else {
            Accumulation::Exact
        }
    }

    pub fn snapshot(&self) -> FixedDurationHistogramSnapshot<N> {
        FixedDurationHistogramSnapshot {
            upper_bounds: self.upper_bounds,
            bucket_counts: self.buckets.map(SaturatingCounter::value),
            overflow_count: self.overflow.value(),
            count: self.count.value(),
            sum: self.sum.value(),
            fidelity_lost: self.fidelity_lost,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn histogram() -> FixedDurationHistogram<3> {
        FixedDurationHistogram::try_new([
            Duration::from_millis(1),
            Duration::from_millis(5),
            Duration::from_millis(10),
        ])
        .unwrap()
    }

    #[test]
    fn bounds_are_nonempty_strictly_ordered_and_closed() {
        assert_eq!(
            FixedDurationHistogram::<0>::try_new([]),
            Err(FixedDurationHistogramBuildError::EmptyBounds)
        );
        assert_eq!(
            FixedDurationHistogram::try_new([Duration::from_millis(1), Duration::from_millis(1),]),
            Err(FixedDurationHistogramBuildError::BoundsNotStrictlyIncreasing)
        );
        let mut histogram = histogram();
        for value in [
            Duration::ZERO,
            Duration::from_millis(1),
            Duration::from_millis(5),
            Duration::from_millis(10),
            Duration::from_millis(11),
        ] {
            assert_eq!(histogram.observe(value), Accumulation::Exact);
        }
        let snapshot = histogram.snapshot();
        assert_eq!(snapshot.bucket_counts, [2, 1, 1]);
        assert_eq!(snapshot.overflow_count, 1);
        assert_eq!(snapshot.count, 5);
        assert_eq!(snapshot.sum, Duration::from_millis(27));
        assert!(!snapshot.fidelity_lost);
    }

    #[test]
    fn exact_folds_are_permutation_invariant_and_partition_the_population() {
        let values = [0_u64, 1, 2, 5, 6, 10, 11, 100];
        let mut forward = histogram();
        let mut reverse = histogram();
        for value in values {
            assert_eq!(
                forward.observe(Duration::from_millis(value)),
                Accumulation::Exact
            );
        }
        for value in values.into_iter().rev() {
            assert_eq!(
                reverse.observe(Duration::from_millis(value)),
                Accumulation::Exact
            );
        }
        let forward = forward.snapshot();
        assert_eq!(forward, reverse.snapshot());
        assert_eq!(
            forward.bucket_counts.into_iter().sum::<u64>() + forward.overflow_count,
            forward.count
        );
    }

    #[test]
    fn every_ordered_small_profile_obeys_monotone_count_and_single_bucket_laws() {
        for first in 0_u64..4 {
            for second in first + 1..6 {
                let mut histogram = FixedDurationHistogram::try_new([
                    Duration::from_nanos(first),
                    Duration::from_nanos(second),
                ])
                .unwrap();
                for offered in 0_u64..8 {
                    let before = histogram.snapshot();
                    assert_eq!(
                        histogram.observe(Duration::from_nanos(offered)),
                        Accumulation::Exact
                    );
                    let after = histogram.snapshot();
                    assert_eq!(after.count, before.count + 1);
                    let bucket_delta = after
                        .bucket_counts
                        .into_iter()
                        .zip(before.bucket_counts)
                        .map(|(after, before)| after - before)
                        .sum::<u64>()
                        + after.overflow_count
                        - before.overflow_count;
                    assert_eq!(bucket_delta, 1);
                    assert!(after.sum >= before.sum);
                }
            }
        }
    }

    #[test]
    fn arithmetic_fidelity_loss_is_sticky_and_stops_claiming_new_knowledge() {
        let mut histogram = histogram();
        histogram.count = SaturatingCounter::from_value(u64::MAX);
        assert_eq!(
            histogram.observe(Duration::from_nanos(1)),
            Accumulation::SaturatedNow
        );
        let saturated = histogram.snapshot();
        assert!(saturated.fidelity_lost);
        assert_eq!(saturated.count, u64::MAX);
        for value in [Duration::ZERO, Duration::MAX] {
            assert_eq!(histogram.observe(value), Accumulation::AlreadySaturated);
            assert_eq!(histogram.snapshot(), saturated);
        }
    }

    #[test]
    fn sum_overflow_also_marks_whole_distribution_fidelity_loss() {
        let mut histogram = histogram();
        histogram.sum = DurationAccumulator::from_value(Duration::MAX);
        assert_eq!(
            histogram.observe(Duration::from_nanos(1)),
            Accumulation::SaturatedNow
        );
        let snapshot = histogram.snapshot();
        assert_eq!(snapshot.sum, Duration::MAX);
        assert!(snapshot.fidelity_lost);
    }
}
