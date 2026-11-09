//! Deterministic inputs for engine and adapter tests.

use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Exhausted;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct LogicalClock(u64);

impl LogicalClock {
    pub fn new(now: u64) -> Self {
        Self(now)
    }

    pub fn now(self) -> u64 {
        self.0
    }

    pub fn advance(&mut self, amount: u64) -> Result<u64, Exhausted> {
        let next = self.0.checked_add(amount).ok_or(Exhausted)?;
        self.0 = next;
        Ok(next)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DeterministicIds(u64);

impl DeterministicIds {
    pub fn new(next: u64) -> Self {
        Self(next)
    }

    pub fn issue(&mut self) -> Result<u64, Exhausted> {
        let issued = self.0;
        self.0 = issued.checked_add(1).ok_or(Exhausted)?;
        Ok(issued)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FaultPlan {
    points: VecDeque<String>,
}

impl FaultPlan {
    pub fn new(points: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            points: points.into_iter().map(Into::into).collect(),
        }
    }

    /// Trips only when `point` is the next planned fault.
    pub fn trip(&mut self, point: &str) -> bool {
        if self.points.front().is_some_and(|next| next == point) {
            self.points.pop_front();
            true
        } else {
            false
        }
    }

    pub fn remaining(&self) -> impl ExactSizeIterator<Item = &str> {
        self.points.iter().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_clock_advance_preserves_time() {
        let mut clock = LogicalClock::new(u64::MAX);
        assert_eq!(clock.advance(1), Err(Exhausted));
        assert_eq!(clock.now(), u64::MAX);
    }

    #[test]
    fn fault_plan_preserves_order_and_ignores_other_points() {
        let mut plan = FaultPlan::new(["before", "after"]);
        assert!(!plan.trip("during"));
        assert!(plan.trip("before"));
        assert_eq!(plan.remaining().collect::<Vec<_>>(), ["after"]);
    }
}
