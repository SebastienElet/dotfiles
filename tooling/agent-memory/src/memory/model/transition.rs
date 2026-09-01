use super::{MemoryEntry, Status, TransitionVerdict, UtcTimestamp};

impl MemoryEntry {
    pub(crate) fn into_transition(
        mut self,
        to: Status,
        at: UtcTimestamp,
        verdict: TransitionVerdict,
        reason: String,
    ) -> Self {
        self.data.status = to;
        self.data.transition = Some(super::EntryTransition::new(
            Status::Active,
            to,
            at,
            verdict,
            reason,
        ));
        self
    }
}
