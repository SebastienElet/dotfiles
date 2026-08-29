use super::{
    InjectedMemory, OmissionEffect, OmittedMemory, RetrievalContext, RetrievalReport,
    RetrievalRequest, SourceSummary,
};
use crate::memory::clock::timestamp;
use crate::memory::{
    EntryScope, MemoryEntry, MemoryError, OracleContext, OracleEvaluation, OracleVerdict,
    SelectedMemory, SourceKind, Status, TransitionVerdict, evaluate_oracle,
};
use url::Url;

pub fn retrieve(request: RetrievalRequest<'_>, context: RetrievalContext<'_>) -> RetrievalReport {
    let extra = request.selection.selected.len().saturating_sub(5);
    let mut report = RetrievalReport {
        injected: Vec::new(),
        omitted: request
            .selection
            .diagnostics
            .iter()
            .map(|diagnostic| omission(&diagnostic.entry_id, &diagnostic.check, None))
            .collect(),
        omitted_by_limit: request.selection.omitted_by_limit + extra,
    };
    for selected in request.selection.selected.iter().take(5) {
        retrieve_selected(selected, &request, &context, &mut report);
    }
    report
}

fn retrieve_selected(
    selected: &SelectedMemory,
    request: &RetrievalRequest<'_>,
    context: &RetrievalContext<'_>,
    report: &mut RetrievalReport,
) {
    let entry = match context.store.load_selected(selected) {
        Ok(entry) => entry,
        Err(error) => {
            report
                .omitted
                .push(omission(&selected.entry_id, error.code(), None));
            return;
        }
    };
    if entry.status() != Status::Active || !in_scope(entry.scope(), request) {
        report
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
                report
                    .omitted
                    .push(omission(entry.id().as_str(), "selection_stale", None));
                return;
            }
            Err(error) => {
                report
                    .omitted
                    .push(omission(entry.id().as_str(), error.code(), None));
                return;
            }
        }
    }
    apply_evaluation(entry, evaluation, context, report);
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
    report: &mut RetrievalReport,
) {
    match evaluation.verdict() {
        OracleVerdict::Valid => match injected(&entry, &evaluation) {
            Some(injected) => report.injected.push(injected),
            None => report
                .omitted
                .push(omission(entry.id().as_str(), "oracle_unavailable", None)),
        },
        OracleVerdict::Invalid => invalidate(entry, &evaluation, context, report),
        OracleVerdict::Unavailable => report.omitted.push(omission(
            entry.id().as_str(),
            "oracle_unavailable",
            Some(entry.oracle().fallback_question()),
        )),
        OracleVerdict::NeedsConfirmation => report.omitted.push(omission(
            entry.id().as_str(),
            "oracle_needs_confirmation",
            Some(entry.oracle().fallback_question()),
        )),
    }
}

fn injected(entry: &MemoryEntry, evaluation: &OracleEvaluation) -> Option<InjectedMemory> {
    let validated_at = timestamp(evaluation.validated_at()?)?;
    let now = timestamp(evaluation.evaluated_at())?;
    let age = now.duration_since(validated_at);
    let verdict_age_milliseconds = u64::try_from(age.as_millis()).ok()?;
    Some(InjectedMemory {
        id: entry.id().as_str().to_owned(),
        kind: entry.kind(),
        statement: entry.statement().as_str().to_owned(),
        sources: source_summaries(entry)?,
        verdict_age_milliseconds,
    })
}

fn source_summaries(entry: &MemoryEntry) -> Option<Vec<SourceSummary>> {
    entry
        .proof()
        .sources()
        .iter()
        .map(|source| match source.kind() {
            SourceKind::GitFile => Some(SourceSummary::with_locator(
                SourceKind::GitFile,
                source.locator(),
            )),
            SourceKind::OfficialUrl => {
                let mut url = Url::parse(source.locator()).ok()?;
                url.set_query(None);
                url.set_fragment(None);
                Some(SourceSummary::with_locator(SourceKind::OfficialUrl, url))
            }
            SourceKind::LocalFile => Some(SourceSummary::redacted(SourceKind::LocalFile)),
            SourceKind::UserDecision => Some(SourceSummary::redacted(SourceKind::UserDecision)),
        })
        .collect()
}

fn invalidate(
    entry: MemoryEntry,
    evaluation: &OracleEvaluation,
    context: &RetrievalContext<'_>,
    report: &mut RetrievalReport,
) {
    let id = entry.id().as_str().to_owned();
    let reason = entry.oracle().invalidated_outcome().to_owned();
    let terminal = entry.into_transition(
        Status::Invalidated,
        evaluation.evaluated_at().clone(),
        TransitionVerdict::Invalid,
        reason,
    );
    let code = match context.store.replace_active(&terminal) {
        Ok(_) => "oracle_invalidated",
        Err(error) => error.code(),
    };
    report.omitted.push(omission(&id, code, None));
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
