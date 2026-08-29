use super::CliFailure;
use super::arguments::{Command, ConfirmArguments, HumanStatus};
use super::boundary::read_required;
use crate::{
    AdmissionAuthorization, AdmissionContext, AdmissionResult, EntryScope, HumanConclusion, Index,
    MemoryError, MemoryKind, MemoryRoot, OracleEnvironment, RetrievalContext, RetrievalRequest,
    SearchRequest, SourceContext, Status, Store, SystemClock, SystemProcessRunner,
    TransitionContext, admit, confirm, resolve_project, retrieve, search,
};
use serde_json::{Value, json};
use std::env;
use std::io::Read;

pub(super) fn dispatch(command: Command, input: &mut dyn Read) -> Result<Value, CliFailure> {
    match command {
        Command::Admit(arguments) => {
            let _ = arguments.format;
            admit_command(&read_required(input)?)
        }
        Command::Retrieve(arguments) => {
            let _ = (arguments.query_stdin, arguments.format);
            retrieve_command(&read_required(input)?)
        }
        Command::Confirm(arguments) => confirm_command(arguments, &read_required(input)?),
        Command::Audit(arguments) => {
            let _ = arguments.format;
            audit_command(arguments.include_terminal)
        }
        Command::Hook(arguments) => {
            let _ = arguments.agent;
            Err(CliFailure::from_memory(MemoryError::new(
                "adapter_unavailable",
                "agent",
            )))
        }
    }
}

fn admit_command(bytes: &[u8]) -> Result<Value, CliFailure> {
    let store = open_store()?;
    let cwd = current_directory()?;
    let clock = SystemClock;
    let processes = SystemProcessRunner;
    let result = admit(
        bytes,
        AdmissionContext {
            store: &store,
            cwd: &cwd,
            clock: &clock,
            processes: &processes,
            authorization: AdmissionAuthorization::ExplicitRequest,
        },
    )
    .map_err(CliFailure::from_memory)?;
    admission_json(result)
}

fn admission_json(result: AdmissionResult) -> Result<Value, CliFailure> {
    match result {
        AdmissionResult::Stored {
            id,
            index_rebuild_required,
        } => Ok(json!({
            "status": "stored",
            "id": id.as_str(),
            "index_rebuild_required": index_rebuild_required,
        })),
        AdmissionResult::Duplicate { id } => Ok(json!({"status": "duplicate", "id": id.as_str()})),
        AdmissionResult::Rejected { error } => Err(CliFailure::from_memory(error)),
        AdmissionResult::Conflict { error, .. } => Err(CliFailure::conflict(error)),
    }
}

fn retrieve_command(bytes: &[u8]) -> Result<Value, CliFailure> {
    let query = std::str::from_utf8(bytes)
        .map_err(|_| CliFailure::from_memory(MemoryError::new("invalid_utf8", "query")))?;
    if query.trim().is_empty() {
        return Err(CliFailure::from_memory(MemoryError::new(
            "empty_query",
            "query",
        )));
    }
    let store = open_store()?;
    let cwd = current_directory()?;
    let processes = SystemProcessRunner;
    let project = resolve_project(&cwd, &processes).map_err(CliFailure::from_memory)?;
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
    let sources = SourceContext::new(&cwd, &processes, &processes);
    let clock = SystemClock;
    let report = retrieve(
        RetrievalRequest::new(&selection, project.key(), true),
        RetrievalContext::new(
            &store,
            &clock,
            &sources,
            OracleEnvironment::new(env::consts::OS, env::consts::ARCH),
        ),
    );
    serde_json::to_value(report).map_err(|_| output_failure())
}

fn confirm_command(arguments: ConfirmArguments, bytes: &[u8]) -> Result<Value, CliFailure> {
    let reason = std::str::from_utf8(bytes)
        .map_err(|_| CliFailure::from_memory(MemoryError::new("invalid_utf8", "reason")))?;
    let conclusion = conclusion(arguments.status, reason).map_err(CliFailure::from_memory)?;
    let store = open_store()?;
    let result = confirm(
        &arguments.id,
        conclusion,
        TransitionContext::new(&store, &SystemClock),
    )
    .map_err(CliFailure::from_memory)?;
    Ok(json!({
        "status": status_name(result.status()),
        "index_rebuild_required": result.index_rebuild_required(),
    }))
}

fn conclusion(status: HumanStatus, reason: &str) -> Result<HumanConclusion, MemoryError> {
    match status {
        HumanStatus::Achieved => HumanConclusion::goal_achieved(reason),
        HumanStatus::Abandoned => HumanConclusion::goal_abandoned(reason),
        HumanStatus::Superseded => HumanConclusion::decision_superseded(reason),
        HumanStatus::Resolved => HumanConclusion::unknown_resolved(reason),
        HumanStatus::Confirmed => HumanConclusion::assumption_confirmed(reason),
    }
}

fn audit_command(include_terminal: bool) -> Result<Value, CliFailure> {
    let store = open_store()?;
    let listing = store.list().map_err(CliFailure::from_memory)?;
    let entries = listing
        .entries()
        .iter()
        .filter(|entry| include_terminal || entry.status() == Status::Active)
        .map(|entry| {
            json!({
                "id": entry.id().as_str(),
                "kind": kind_name(entry.kind()),
                "status": status_name(entry.status()),
                "scope": scope_json(entry.scope()),
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "entries": entries,
        "index_rebuild_required": listing.index_rebuild_required(),
    }))
}

fn open_store() -> Result<Store, CliFailure> {
    let root = MemoryRoot::from_environment().map_err(CliFailure::from_memory)?;
    Store::open(root).map_err(CliFailure::from_memory)
}

fn current_directory() -> Result<std::path::PathBuf, CliFailure> {
    env::current_dir()
        .map_err(|_| CliFailure::from_memory(MemoryError::new("scope_unavailable", "scope")))
}

fn scope_json(scope: &EntryScope) -> Value {
    match scope {
        EntryScope::Project(key) => json!({"type": "project", "key": key.as_str()}),
        EntryScope::User => json!({"type": "user"}),
    }
}

fn kind_name(kind: MemoryKind) -> &'static str {
    match kind {
        MemoryKind::Goal => "goal",
        MemoryKind::Decision => "decision",
        MemoryKind::Evidence => "evidence",
        MemoryKind::Invariant => "invariant",
        MemoryKind::Unknown => "unknown",
        MemoryKind::Assumption => "assumption",
    }
}

fn status_name(status: Status) -> &'static str {
    match status {
        Status::Active => "active",
        Status::Achieved => "achieved",
        Status::Abandoned => "abandoned",
        Status::Superseded => "superseded",
        Status::Invalidated => "invalidated",
        Status::Resolved => "resolved",
        Status::Confirmed => "confirmed",
    }
}

fn output_failure() -> CliFailure {
    CliFailure {
        exit: 4,
        code: "output_unavailable",
        field: "stdout",
    }
}
