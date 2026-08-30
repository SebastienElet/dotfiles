use std::time::{Duration, Instant};

const MAX_CLEANUP_RESERVE: Duration = Duration::from_secs(1);

#[derive(Clone, Copy)]
pub(super) struct Deadlines {
    hard: Instant,
    work: Instant,
}

impl Deadlines {
    pub(super) fn new(hard: Instant) -> Self {
        let remaining = hard.saturating_duration_since(Instant::now());
        let reserve = MAX_CLEANUP_RESERVE.min(remaining / 4);
        Self {
            hard,
            work: hard.checked_sub(reserve).unwrap_or(hard),
        }
    }

    pub(super) fn hard(&self) -> Instant {
        self.hard
    }

    pub(super) fn work_expired(&self) -> bool {
        Instant::now() >= self.work
    }

    pub(super) fn remaining_work(&self) -> Duration {
        self.work.saturating_duration_since(Instant::now())
    }
}
