use crate::{
    AdmissionAuthorization, AdmissionResult, Clock, MemoryError, ProcessRunner, ScopeDraft,
    SourceContext, Store, ValidatedDraft, parse_draft, resolve_project, resolve_sources,
    validate_draft,
};
use std::path::Path;

pub struct AdmissionContext<'a> {
    pub store: &'a Store,
    pub cwd: &'a Path,
    pub clock: &'a dyn Clock,
    pub processes: &'a dyn ProcessRunner,
    pub authorization: AdmissionAuthorization,
}

pub(crate) struct PreparedAdmission {
    resolved: crate::ResolvedDraft,
    project: Option<crate::ProjectScope>,
}

pub fn admit(bytes: &[u8], context: AdmissionContext<'_>) -> Result<AdmissionResult, MemoryError> {
    let draft = match prepare_admission(bytes, context.authorization) {
        Ok(draft) => draft,
        Err(error) => return Ok(AdmissionResult::Rejected { error }),
    };
    let prepared = match resolve_admission(draft, context.cwd, context.processes) {
        Ok(prepared) => prepared,
        Err(error) => return Ok(AdmissionResult::Rejected { error }),
    };
    let sources = SourceContext::new(context.cwd, context.processes, context.processes);
    admit_prepared(prepared, context.store, context.clock, &sources)
}

pub fn prepare_admission(
    bytes: &[u8],
    authorization: AdmissionAuthorization,
) -> Result<ValidatedDraft, MemoryError> {
    validate_draft(parse_draft(bytes)?, authorization)
}

pub(crate) fn resolve_admission(
    draft: ValidatedDraft,
    cwd: &Path,
    processes: &dyn ProcessRunner,
) -> Result<PreparedAdmission, MemoryError> {
    let project = match draft.scope() {
        ScopeDraft::Project => Some(resolve_project(cwd, processes)?),
        ScopeDraft::User => None,
    };
    let sources = SourceContext::new(cwd, processes, processes);
    let resolved = resolve_sources(draft, &sources)?;
    Ok(PreparedAdmission { resolved, project })
}

pub(crate) fn admit_prepared(
    prepared: PreparedAdmission,
    store: &Store,
    clock: &dyn Clock,
    sources: &SourceContext<'_>,
) -> Result<AdmissionResult, MemoryError> {
    Ok(store.admit(
        prepared.resolved,
        prepared.project.as_ref(),
        &clock.now(),
        sources,
    ))
}
