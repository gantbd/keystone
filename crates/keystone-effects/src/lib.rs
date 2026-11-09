//! Lawful accumulated knowledge about an external effect.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectKnowledge {
    Unknown,
    Failed,
    Completed,
    Conflict,
}

impl EffectKnowledge {
    pub const ALL: [Self; 4] = [Self::Unknown, Self::Failed, Self::Completed, Self::Conflict];

    pub const fn bottom() -> Self {
        Self::Unknown
    }

    pub const fn top() -> Self {
        Self::Conflict
    }

    /// Partial order induced by [`EffectKnowledge::join`].
    pub fn leq(self, other: Self) -> bool {
        self.join(other) == other
    }

    pub fn join(self, other: Self) -> Self {
        use EffectKnowledge::*;
        match (self, other) {
            (Unknown, value) | (value, Unknown) => value,
            (Conflict, _) | (_, Conflict) => Conflict,
            (Failed, Failed) => Failed,
            (Completed, Completed) => Completed,
            (Failed, Completed) | (Completed, Failed) => Conflict,
        }
    }

    pub fn meet(self, other: Self) -> Self {
        use EffectKnowledge::*;
        match (self, other) {
            (Conflict, value) | (value, Conflict) => value,
            (Unknown, _) | (_, Unknown) => Unknown,
            (Failed, Failed) => Failed,
            (Completed, Completed) => Completed,
            (Failed, Completed) | (Completed, Failed) => Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhaustive_bounded_lattice_laws_hold() {
        for a in EffectKnowledge::ALL {
            assert_eq!(a.join(EffectKnowledge::bottom()), a);
            assert_eq!(a.meet(EffectKnowledge::top()), a);
            assert!(EffectKnowledge::bottom().leq(a));
            assert!(a.leq(EffectKnowledge::top()));
            assert_eq!(a.join(a), a);
            assert_eq!(a.meet(a), a);
            for b in EffectKnowledge::ALL {
                assert_eq!(a.join(b), b.join(a));
                assert_eq!(a.meet(b), b.meet(a));
                assert_eq!(a.join(a.meet(b)), a);
                assert_eq!(a.meet(a.join(b)), a);
                for c in EffectKnowledge::ALL {
                    assert_eq!(a.join(b).join(c), a.join(b.join(c)));
                    assert_eq!(a.meet(b).meet(c), a.meet(b.meet(c)));
                }
            }
        }
        assert!(!EffectKnowledge::Failed.leq(EffectKnowledge::Completed));
        assert!(!EffectKnowledge::Completed.leq(EffectKnowledge::Failed));
    }
}
