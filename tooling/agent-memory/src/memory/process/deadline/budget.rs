use std::time::{Duration, Instant};

const MAX_CLEANUP_RESERVE: Duration = Duration::from_secs(1);

#[derive(Clone, Copy)]
pub(super) struct ProcessBudget {
    cleanup_deadline: Instant,
    work_cutoff: Instant,
}

impl ProcessBudget {
    pub(super) fn new(cleanup_deadline: Instant) -> Self {
        let remaining = cleanup_deadline.saturating_duration_since(Instant::now());
        let reserve = MAX_CLEANUP_RESERVE.min(remaining / 4);
        Self {
            cleanup_deadline,
            work_cutoff: cleanup_deadline
                .checked_sub(reserve)
                .unwrap_or(cleanup_deadline),
        }
    }

    pub(super) fn cleanup_deadline(&self) -> Instant {
        self.cleanup_deadline
    }

    pub(super) fn work_cutoff_observed_expired(&self) -> bool {
        Instant::now() >= self.work_cutoff
    }

    pub(super) fn remaining_work_at_observation(&self) -> Duration {
        self.work_cutoff.saturating_duration_since(Instant::now())
    }
}
