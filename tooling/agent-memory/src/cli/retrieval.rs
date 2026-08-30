use super::CliFailure;
use crate::{
    DeadlineProcessRunner, Index, MemoryRoot, OracleEnvironment, RetrievalContext, RetrievalReport,
    RetrievalRequest, SearchRequest, SourceContext, Store, SystemClock, SystemProcessRunner,
    resolve_project, retrieve, retrieve_for_injection, search,
};
use std::env;
use std::path::Path;
use std::time::Instant;

pub(super) fn report(query: &str, cwd: &Path) -> Result<RetrievalReport, CliFailure> {
    report_with_processes(
        query,
        cwd,
        RetrievalMode::Report,
        &SystemProcessRunner,
        None,
    )
}

pub(super) fn injection_report(
    query: &str,
    cwd: &Path,
    deadline: Instant,
) -> Result<RetrievalReport, CliFailure> {
    let processes = DeadlineProcessRunner::new(deadline);
    report_with_processes(
        query,
        cwd,
        RetrievalMode::Injection,
        &processes,
        Some(deadline),
    )
}

#[derive(Clone, Copy)]
enum RetrievalMode {
    Report,
    Injection,
}

fn report_with_processes(
    query: &str,
    cwd: &Path,
    mode: RetrievalMode,
    processes: &dyn crate::ProcessRunner,
    deadline: Option<Instant>,
) -> Result<RetrievalReport, CliFailure> {
    let project = resolve_project(cwd, processes).map_err(CliFailure::from_memory)?;
    let Some(store) = open_store()? else {
        return Ok(empty_report());
    };
    let index = Index::load_or_rebuild(&store).map_err(CliFailure::from_memory)?;
    let selection = search(
        &index.index,
        SearchRequest {
            query,
            project_key: project.key(),
            include_user: true,
            limit: 5,
        },
    );
    let sources = SourceContext::new(cwd, processes, processes);
    let request = RetrievalRequest::new(&selection, project.key(), true);
    let mut context = RetrievalContext::new(
        &store,
        &SystemClock,
        &sources,
        OracleEnvironment::new(env::consts::OS, env::consts::ARCH),
    );
    if let Some(deadline) = deadline {
        context = context.with_deadline(deadline);
    }
    match mode {
        RetrievalMode::Report => Ok(retrieve(request, context)),
        RetrievalMode::Injection => {
            retrieve_for_injection(request, context).map_err(CliFailure::from_memory)
        }
    }
}

fn open_store() -> Result<Option<Store>, CliFailure> {
    let root = MemoryRoot::from_environment().map_err(CliFailure::from_memory)?;
    Store::open_for_retrieval(root).map_err(CliFailure::from_memory)
}

fn empty_report() -> RetrievalReport {
    RetrievalReport {
        injected: Vec::new(),
        omitted: Vec::new(),
        omitted_by_limit: 0,
    }
}
