//! Explicit ownership obligations for cooperative shutdown.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecyclePhase {
    Open,
    Draining,
    Closed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LifecycleError<I> {
    Duplicate(I),
    Unknown(I),
    ObligationLimit,
    NotOpen,
    OutstandingObligations(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObligationLedger<I> {
    phase: LifecyclePhase,
    limit: usize,
    obligations: BTreeSet<I>,
}

impl<I: Ord> ObligationLedger<I> {
    pub fn new(limit: usize) -> Self {
        Self {
            phase: LifecyclePhase::Open,
            limit,
            obligations: BTreeSet::new(),
        }
    }

    pub fn phase(&self) -> LifecyclePhase {
        self.phase
    }

    pub fn obligations(&self) -> impl ExactSizeIterator<Item = &I> {
        self.obligations.iter()
    }

    pub fn register(&mut self, id: I) -> Result<(), LifecycleError<I>> {
        if self.phase != LifecyclePhase::Open {
            return Err(LifecycleError::NotOpen);
        }
        if self.obligations.contains(&id) {
            return Err(LifecycleError::Duplicate(id));
        }
        if self.obligations.len() >= self.limit {
            return Err(LifecycleError::ObligationLimit);
        }
        self.obligations.insert(id);
        Ok(())
    }

    pub fn discharge(&mut self, id: I) -> Result<(), LifecycleError<I>> {
        if !self.obligations.remove(&id) {
            return Err(LifecycleError::Unknown(id));
        }
        Ok(())
    }

    pub fn begin_draining(&mut self) {
        if self.phase == LifecyclePhase::Open {
            self.phase = LifecyclePhase::Draining;
        }
    }

    pub fn close(&mut self) -> Result<(), LifecycleError<I>> {
        if !self.obligations.is_empty() {
            return Err(LifecycleError::OutstandingObligations(
                self.obligations.len(),
            ));
        }
        self.phase = LifecyclePhase::Closed;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutdown_retains_obligations_until_explicit_discharge() {
        let mut ledger = ObligationLedger::new(1);
        ledger.register("work").unwrap();
        ledger.begin_draining();
        assert_eq!(
            ledger.close(),
            Err(LifecycleError::OutstandingObligations(1))
        );
        assert_eq!(ledger.obligations().copied().collect::<Vec<_>>(), ["work"]);
        ledger.discharge("work").unwrap();
        ledger.close().unwrap();
        assert_eq!(ledger.phase(), LifecyclePhase::Closed);
    }

    #[test]
    fn draining_refuses_new_work_without_changing_inventory() {
        let mut ledger = ObligationLedger::new(1);
        ledger.begin_draining();
        assert_eq!(ledger.register("late"), Err(LifecycleError::NotOpen));
        assert_eq!(ledger.obligations().len(), 0);
    }
}
