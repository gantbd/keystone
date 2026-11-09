//! Executable cross-project qualification evidence.
//!
//! This crate is not a runtime adapter and must never enter production dependency
//! graphs. It proves selected semantic correspondence across independently owned
//! projects.

use chronos_engine::{BoundedLattice, EffectKnowledge as ChronosKnowledge};
use keystone_effects::EffectKnowledge as KeystoneKnowledge;

pub fn from_chronos(value: ChronosKnowledge) -> KeystoneKnowledge {
    match value {
        ChronosKnowledge::Unknown => KeystoneKnowledge::Unknown,
        ChronosKnowledge::Failed => KeystoneKnowledge::Failed,
        ChronosKnowledge::Completed => KeystoneKnowledge::Completed,
        ChronosKnowledge::Conflict => KeystoneKnowledge::Conflict,
    }
}

pub fn to_chronos(value: KeystoneKnowledge) -> ChronosKnowledge {
    match value {
        KeystoneKnowledge::Unknown => ChronosKnowledge::Unknown,
        KeystoneKnowledge::Failed => ChronosKnowledge::Failed,
        KeystoneKnowledge::Completed => ChronosKnowledge::Completed,
        KeystoneKnowledge::Conflict => ChronosKnowledge::Conflict,
    }
}

pub fn chronos_join(left: ChronosKnowledge, right: ChronosKnowledge) -> ChronosKnowledge {
    left.join(right)
}

pub fn chronos_meet(left: ChronosKnowledge, right: ChronosKnowledge) -> ChronosKnowledge {
    left.meet(right)
}

#[cfg(test)]
mod tests {
    use super::*;
    use moira_core::EffectKnowledgeV1 as MoiraKnowledge;

    const CHRONOS_VALUES: [ChronosKnowledge; 4] = [
        ChronosKnowledge::Unknown,
        ChronosKnowledge::Failed,
        ChronosKnowledge::Completed,
        ChronosKnowledge::Conflict,
    ];

    #[test]
    fn mapping_is_a_bijection_preserving_bottom_and_top() {
        for chronos in CHRONOS_VALUES {
            assert_eq!(to_chronos(from_chronos(chronos)), chronos);
        }
        assert_eq!(
            from_chronos(ChronosKnowledge::bottom()),
            KeystoneKnowledge::bottom()
        );
        assert_eq!(
            from_chronos(ChronosKnowledge::top()),
            KeystoneKnowledge::top()
        );
    }

    #[test]
    fn every_join_meet_and_order_result_matches_chronos() {
        for left in CHRONOS_VALUES {
            for right in CHRONOS_VALUES {
                let keystone_left = from_chronos(left);
                let keystone_right = from_chronos(right);
                assert_eq!(
                    from_chronos(chronos_join(left, right)),
                    keystone_left.join(keystone_right)
                );
                assert_eq!(
                    from_chronos(chronos_meet(left, right)),
                    keystone_left.meet(keystone_right)
                );
                assert_eq!(left.leq(right), keystone_left.leq(keystone_right));
            }
        }
    }

    #[test]
    fn moira_consumes_the_qualified_keystone_type_directly() {
        let accumulated: MoiraKnowledge = MoiraKnowledge::Completed.join(MoiraKnowledge::Failed);
        assert_eq!(accumulated, KeystoneKnowledge::Conflict);
    }
}
