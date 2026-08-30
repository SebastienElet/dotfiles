use super::CliFailure;
use crate::{
    Index, MemoryRoot, OracleEnvironment, RetrievalContext, RetrievalReport, RetrievalRequest,
    SearchRequest, SourceContext, Store, SystemClock, SystemProcessRunner, resolve_project,
    retrieve, search,
};
use std::env;
use std::path::Path;

pub(super) fn report(query: &str, cwd: &Path) -> Result<RetrievalReport, CliFailure> {
    let processes = SystemProcessRunner;
    let project = resolve_project(cwd, &processes).map_err(CliFailure::from_memory)?;
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
    let sources = SourceContext::new(cwd, &processes, &processes);
    Ok(retrieve(
        RetrievalRequest::new(&selection, project.key(), true),
        RetrievalContext::new(
            &store,
            &SystemClock,
            &sources,
            OracleEnvironment::new(env::consts::OS, env::consts::ARCH),
        ),
    ))
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
