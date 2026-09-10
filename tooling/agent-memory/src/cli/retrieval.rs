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
    let project =
        resolve_project(cwd, processes).map_err(|error| CliFailure::from_memory(&error))?;
    let Some(store) = open_store()? else {
        return Ok(empty_report());
    };
    let index = Index::load_or_rebuild(&store).map_err(|error| CliFailure::from_memory(&error))?;
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
        RetrievalMode::Report => Ok(retrieve(request, &context)),
        RetrievalMode::Injection => retrieve_for_injection(request, &context)
            .map_err(|error| CliFailure::from_memory(&error)),
    }
}

fn open_store() -> Result<Option<Store>, CliFailure> {
    let root = MemoryRoot::from_environment().map_err(|error| CliFailure::from_memory(&error))?;
    Store::open_for_retrieval(&root).map_err(|error| CliFailure::from_memory(&error))
}

const fn empty_report() -> RetrievalReport {
    RetrievalReport {
        injected: Vec::new(),
        omitted: Vec::new(),
        omitted_by_limit: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProcessOutput;
    use std::ffi::{OsStr, OsString};
    use std::io;

    struct FailingGit(io::ErrorKind);

    impl crate::ProcessRunner for FailingGit {
        fn run(
            &self,
            _program: &OsStr,
            _arguments: &[OsString],
            _current_directory: Option<&Path>,
        ) -> io::Result<ProcessOutput> {
            Err(io::Error::new(self.0, "process_unavailable"))
        }
    }

    #[test]
    fn hook_retrieval_maps_project_lookup_timeout_to_exit_four()
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let error = report_with_processes(
            "durable memory",
            Path::new("/"),
            RetrievalMode::Injection,
            &FailingGit(io::ErrorKind::TimedOut),
            Some(Instant::now() + std::time::Duration::from_secs(1)),
        )
        .err()
        .ok_or("expected operation failure")?;

        assert_eq!(error.exit, 4);
        assert_eq!(error.code, "scope_unavailable");
        Ok(())
    }

    #[test]
    fn hook_retrieval_maps_project_process_failure_to_exit_four()
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let error = report_with_processes(
            "durable memory",
            Path::new("/"),
            RetrievalMode::Injection,
            &FailingGit(io::ErrorKind::Other),
            Some(Instant::now() + std::time::Duration::from_secs(1)),
        )
        .err()
        .ok_or("expected operation failure")?;

        assert_eq!(error.exit, 4);
        assert_eq!(error.code, "scope_unavailable");
        Ok(())
    }
}
