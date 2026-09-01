mod projection;

use super::{OmissionEffect, OmittedMemory, RetrievalContext, RetrievalReport, RetrievalRequest};
use crate::memory::{
    EntryScope, MemoryEntry, MemoryError, OracleContext, OracleEvaluation, OracleVerdict,
    SelectedMemory, Status, TransitionVerdict, evaluate_oracle,
};
use projection::injected;

pub fn retrieve(request: RetrievalRequest<'_>, context: RetrievalContext<'_>) -> RetrievalReport {
    retrieve_outcome(request, context, RetrievalMode::Report).report
}

pub fn retrieve_for_injection(
    request: RetrievalRequest<'_>,
    context: RetrievalContext<'_>,
) -> Result<RetrievalReport, MemoryError> {
    let outcome = retrieve_outcome(request, context, RetrievalMode::Injection);
    outcome.unavailable.map_or(Ok(outcome.report), Err)
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum RetrievalMode {
    Report,
    Injection,
}

struct RetrievalOutcome {
    report: RetrievalReport,
    unavailable: Option<MemoryError>,
}

fn retrieve_outcome(
    request: RetrievalRequest<'_>,
    context: RetrievalContext<'_>,
    mode: RetrievalMode,
) -> RetrievalOutcome {
    let extra = request.selection.selected.len().saturating_sub(5);
    let mut outcome = RetrievalOutcome {
        report: RetrievalReport {
            injected: Vec::new(),
            omitted: request
                .selection
                .diagnostics
                .iter()
                .map(|diagnostic| omission(&diagnostic.entry_id, &diagnostic.check, None))
                .collect(),
            omitted_by_limit: request.selection.omitted_by_limit + extra,
        },
        unavailable: None,
    };
    for selected in request.selection.selected.iter().take(5) {
        if context.deadline_exceeded() {
            outcome.unavailable = Some(MemoryError::unavailable(
                "retrieval_deadline_exceeded",
                "memory",
            ));
            break;
        }
        retrieve_selected(selected, &request, &context, &mut outcome);
        if context.deadline_exceeded() && outcome.unavailable.is_none() {
            outcome.unavailable = Some(MemoryError::unavailable(
                "retrieval_deadline_exceeded",
                "memory",
            ));
        }
        if mode == RetrievalMode::Injection && outcome.unavailable.is_some() {
            break;
        }
    }
    outcome
}

fn retrieve_selected(
    selected: &SelectedMemory,
    request: &RetrievalRequest<'_>,
    context: &RetrievalContext<'_>,
    outcome: &mut RetrievalOutcome,
) {
    let entry = match context.store.load_selected(selected) {
        Ok(entry) => entry,
        Err(error) => {
            omit_error(outcome, &selected.entry_id, error, None);
            return;
        }
    };
    if entry.status() != Status::Active || !in_scope(entry.scope(), request) {
        outcome
            .report
            .omitted
            .push(omission(entry.id().as_str(), "selection_stale", None));
        return;
    }
    let mut oracle = OracleContext::new(
        context.store,
        context.clock,
        context.resolver,
        context.environment.clone(),
    );
    if let Some(answer) = context.proof_valid(entry.id().as_str()) {
        oracle = oracle.with_proof_valid(answer);
    }
    let evaluation = evaluate_oracle(&entry, oracle);
    if evaluation.verdict() == OracleVerdict::Valid {
        match revalidate_before_injection(selected, request, context) {
            Ok(true) => {}
            Ok(false) => {
                outcome
                    .report
                    .omitted
                    .push(omission(entry.id().as_str(), "selection_stale", None));
                return;
            }
            Err(error) => {
                omit_error(outcome, entry.id().as_str(), error, None);
                return;
            }
        }
    }
    apply_evaluation(entry, evaluation, context, outcome);
}

fn revalidate_before_injection(
    selected: &SelectedMemory,
    request: &RetrievalRequest<'_>,
    context: &RetrievalContext<'_>,
) -> Result<bool, MemoryError> {
    let entry = context.store.load_selected(selected)?;
    Ok(entry.status() == Status::Active && in_scope(entry.scope(), request))
}

fn apply_evaluation(
    entry: MemoryEntry,
    evaluation: OracleEvaluation,
    context: &RetrievalContext<'_>,
    outcome: &mut RetrievalOutcome,
) {
    match evaluation.verdict() {
        OracleVerdict::Valid => match injected(&entry, &evaluation) {
            Some(injected) => outcome.report.injected.push(injected),
            None => omit_error(
                outcome,
                entry.id().as_str(),
                MemoryError::unavailable("oracle_unavailable", "oracle"),
                None,
            ),
        },
        OracleVerdict::Invalid => invalidate(entry, &evaluation, context, outcome),
        OracleVerdict::Unavailable => {
            let question = entry.oracle().fallback_question().to_owned();
            omit_error(
                outcome,
                entry.id().as_str(),
                MemoryError::unavailable("oracle_unavailable", "oracle"),
                Some(&question),
            );
        }
        OracleVerdict::NeedsConfirmation => outcome.report.omitted.push(omission(
            entry.id().as_str(),
            "oracle_needs_confirmation",
            Some(entry.oracle().fallback_question()),
        )),
    }
}

fn invalidate(
    entry: MemoryEntry,
    evaluation: &OracleEvaluation,
    context: &RetrievalContext<'_>,
    outcome: &mut RetrievalOutcome,
) {
    let id = entry.id().as_str().to_owned();
    let reason = entry.oracle().invalidated_outcome().to_owned();
    let terminal = entry.into_transition(
        Status::Invalidated,
        evaluation.evaluated_at().clone(),
        TransitionVerdict::Invalid,
        reason,
    );
    match context.store.replace_active(&terminal) {
        Ok(_) => outcome
            .report
            .omitted
            .push(omission(&id, "oracle_invalidated", None)),
        Err(error) => omit_error(outcome, &id, error, None),
    }
}

fn omit_error(
    outcome: &mut RetrievalOutcome,
    entry_id: &str,
    error: MemoryError,
    question: Option<&str>,
) {
    outcome
        .report
        .omitted
        .push(omission(entry_id, error.code(), question));
    if error.class() == crate::MemoryErrorClass::Unavailable && outcome.unavailable.is_none() {
        outcome.unavailable = Some(error);
    }
}

fn in_scope(scope: &EntryScope, request: &RetrievalRequest<'_>) -> bool {
    match scope {
        EntryScope::Project(key) => key == request.project_key,
        EntryScope::User => request.include_user,
    }
}

fn omission(entry_id: &str, code: &str, question: Option<&str>) -> OmittedMemory {
    OmittedMemory {
        id: entry_id.to_owned(),
        code: code.to_owned(),
        question: question.map(str::to_owned),
        effect: OmissionEffect::NotApplied,
    }
}
