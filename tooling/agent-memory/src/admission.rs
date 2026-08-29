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

pub fn admit(bytes: &[u8], context: AdmissionContext<'_>) -> Result<AdmissionResult, MemoryError> {
    let draft = match prepare_admission(bytes, context.authorization) {
        Ok(draft) => draft,
        Err(error) => return Ok(AdmissionResult::Rejected { error }),
    };
    admit_prepared(draft, context)
}

pub fn prepare_admission(
    bytes: &[u8],
    authorization: AdmissionAuthorization,
) -> Result<ValidatedDraft, MemoryError> {
    validate_draft(parse_draft(bytes)?, authorization)
}

pub(crate) fn admit_prepared(
    draft: ValidatedDraft,
    context: AdmissionContext<'_>,
) -> Result<AdmissionResult, MemoryError> {
    let project = match draft.scope() {
        ScopeDraft::Project => match resolve_project(context.cwd, context.processes) {
            Ok(project) => Some(project),
            Err(error) => return Ok(AdmissionResult::Rejected { error }),
        },
        ScopeDraft::User => None,
    };
    let sources = SourceContext::new(context.cwd, context.processes, context.processes);
    let resolved = match resolve_sources(draft, &sources) {
        Ok(resolved) => resolved,
        Err(error) => return Ok(AdmissionResult::Rejected { error }),
    };
    Ok(context
        .store
        .admit(resolved, project.as_ref(), &context.clock.now(), &sources))
}
