use crate::{BundleId, Priority};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleCandidate {
    pub id: BundleId,
    pub priority: Priority,
    pub created_at_ms: u64,
    pub attempts: u32,
}

#[derive(Debug, Clone, Default)]
pub struct BundleScheduler;

impl BundleScheduler {
    pub fn order(&self, candidates: &mut [ScheduleCandidate]) {
        candidates.sort_by(|a, b| {
            priority_rank(b.priority)
                .cmp(&priority_rank(a.priority))
                .then_with(|| a.attempts.cmp(&b.attempts))
                .then_with(|| a.created_at_ms.cmp(&b.created_at_ms))
                .then_with(|| a.id.cmp(&b.id))
        });
    }
}

fn priority_rank(priority: Priority) -> u8 {
    match priority {
        Priority::Routine => 0,
        Priority::Important => 1,
        Priority::Urgent => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urgent_bundles_are_scheduled_first() {
        let mut items = vec![
            ScheduleCandidate {
                id: "routine".into(),
                priority: Priority::Routine,
                created_at_ms: 1,
                attempts: 0,
            },
            ScheduleCandidate {
                id: "urgent".into(),
                priority: Priority::Urgent,
                created_at_ms: 9,
                attempts: 0,
            },
            ScheduleCandidate {
                id: "important".into(),
                priority: Priority::Important,
                created_at_ms: 2,
                attempts: 0,
            },
        ];

        BundleScheduler.order(&mut items);
        assert_eq!(items[0].id, "urgent");
        assert_eq!(items[1].id, "important");
        assert_eq!(items[2].id, "routine");
    }

    #[test]
    fn older_bundle_wins_within_same_priority_and_attempt_count() {
        let mut items = vec![
            ScheduleCandidate {
                id: "new".into(),
                priority: Priority::Important,
                created_at_ms: 20,
                attempts: 1,
            },
            ScheduleCandidate {
                id: "old".into(),
                priority: Priority::Important,
                created_at_ms: 10,
                attempts: 1,
            },
        ];

        BundleScheduler.order(&mut items);
        assert_eq!(items[0].id, "old");
    }
}
