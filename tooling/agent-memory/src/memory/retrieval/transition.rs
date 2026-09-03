use super::{HumanConclusion, TransitionContext, TransitionResult};
use crate::memory::{MemoryError, Status, TransitionVerdict};

pub fn confirm(
    id: &str,
    conclusion: HumanConclusion,
    context: TransitionContext<'_>,
) -> Result<TransitionResult, MemoryError> {
    let entry = context
        .store
        .load(id)?
        .ok_or_else(|| MemoryError::new("entry_not_found", "id"))?;
    if entry.status() != Status::Active {
        return Err(MemoryError::conflict("entry_not_active", "status"));
    }
    let status = conclusion.status_for(entry.kind()).ok_or_else(|| {
        MemoryError::new("invalid_human_conclusion", "conclusion")
            .with_message(human_status_requirement(entry.kind()))
    })?;
    let terminal = entry.into_transition(
        status,
        context.clock.now(),
        TransitionVerdict::Valid,
        conclusion.reason,
    );
    let commit = context.store.replace_active(&terminal)?;
    Ok(TransitionResult {
        status,
        index_rebuild_required: commit.index_rebuild_required(),
    })
}

fn human_status_requirement(kind: crate::MemoryKind) -> &'static str {
    match kind {
        crate::MemoryKind::Goal => {
            "For a goal, the human terminal statuses are achieved and abandoned."
        }
        crate::MemoryKind::Decision => {
            "For a decision, the only human terminal status is superseded."
        }
        crate::MemoryKind::Unknown => "For an unknown, the only human terminal status is resolved.",
        crate::MemoryKind::Assumption => {
            "For an assumption, the only human terminal status is confirmed."
        }
        crate::MemoryKind::Evidence | crate::MemoryKind::Invariant => {
            "Evidence and invariant entries have no human terminal status; do not confirm them."
        }
    }
}
