use agent_memory::{
    AdmissionAuthorization, AdmissionContext, AdmissionResult, MemoryRoot, Store, SystemClock,
    admit,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

struct UnusedProcessRunner;

impl agent_memory::ProcessRunner for UnusedProcessRunner {
    fn run(
        &self,
        _program: &std::ffi::OsStr,
        _arguments: &[std::ffi::OsString],
        _current_directory: Option<&Path>,
    ) -> std::io::Result<agent_memory::ProcessOutput> {
        panic!("user-decision source must not invoke a process")
    }
}

fn draft(retrieval_term: &str) -> Vec<u8> {
    format!(
        "schema_version: 1\nkind: invariant\nstatement: Concurrent durable invariant.\nscope: user\nretrieval_terms:\n  - {retrieval_term}\nproof:\n  summary: An explicit decision establishes the invariant.\n  sources:\n    - kind: user-decision\n      locator: decision:concurrent-memory-test\noracle:\n  human_fallback:\n    question: Does the decision still establish this invariant?\n    valid_when: The decision remains in force.\n  outcomes:\n    valid: The invariant remains established.\n    invalidated: The decision no longer establishes the invariant.\n"
    )
    .into_bytes()
}

#[test]
fn concurrency_worker() {
    let Some(output) = std::env::var_os("AGENT_MEMORY_CONCURRENCY_OUTPUT") else {
        return;
    };
    let ready = PathBuf::from(std::env::var_os("AGENT_MEMORY_CONCURRENCY_READY").unwrap());
    let go = PathBuf::from(std::env::var_os("AGENT_MEMORY_CONCURRENCY_GO").unwrap());
    let variant = std::env::var("AGENT_MEMORY_CONCURRENCY_VARIANT").unwrap();
    let root = MemoryRoot::from_environment().unwrap();
    let store = Store::open(root).unwrap();
    let runner = UnusedProcessRunner;
    let cwd = ready.parent().unwrap();
    fs::write(&ready, b"ready").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !go.exists() {
        assert!(Instant::now() < deadline, "concurrency gate timed out");
        std::thread::sleep(Duration::from_millis(5));
    }
    let outcome = match admit(
        &draft(&variant),
        AdmissionContext {
            store: &store,
            cwd,
            clock: &SystemClock,
            processes: &runner,
            authorization: AdmissionAuthorization::ExplicitRequest,
        },
    )
    .unwrap()
    {
        AdmissionResult::Stored { .. } => "stored",
        AdmissionResult::Duplicate { .. } => "duplicate",
        AdmissionResult::Conflict { .. } => "conflict",
        AdmissionResult::Rejected { .. } => "rejected",
    };
    fs::write(output, format!("{}:{outcome}", std::process::id())).unwrap();
}

#[test]
fn two_processes_admitting_the_same_draft_produce_stored_and_duplicate() {
    let result = compete("same", "same");

    assert_eq!(result.outcomes, ["duplicate", "stored"]);
    assert_ne!(result.processes[0], result.processes[1]);
    assert_eq!(result.entry_count, 1);
}

#[test]
fn two_processes_admitting_divergent_drafts_at_the_same_id_produce_stored_and_conflict() {
    let result = compete("first", "second");

    assert_eq!(result.outcomes, ["conflict", "stored"]);
    assert_ne!(result.processes[0], result.processes[1]);
    assert_eq!(result.entry_count, 1);
}

struct Competition {
    outcomes: Vec<String>,
    processes: Vec<u32>,
    entry_count: usize,
}

fn compete(first: &str, second: &str) -> Competition {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    Store::open(MemoryRoot::new(&root).unwrap()).unwrap();
    let go = fixture.path().join("go");
    let executable = std::env::current_exe().unwrap();
    let variants = [first, second];
    let mut children = Vec::new();
    let mut ready_paths = Vec::new();
    let mut output_paths = Vec::new();
    for (index, variant) in variants.into_iter().enumerate() {
        let ready = fixture.path().join(format!("ready-{index}"));
        let output = fixture.path().join(format!("output-{index}"));
        let child = spawn_worker(&executable, &root, &go, &ready, &output, variant);
        children.push(child);
        ready_paths.push(ready);
        output_paths.push(output);
    }
    wait_until_ready(&ready_paths);
    fs::write(&go, b"go").unwrap();
    for mut child in children {
        assert!(child.wait().unwrap().success());
    }
    let mut outcomes = Vec::new();
    let mut processes = Vec::new();
    for output in output_paths {
        let value = fs::read_to_string(output).unwrap();
        let (process, outcome) = value.split_once(':').unwrap();
        processes.push(process.parse().unwrap());
        outcomes.push(outcome.to_owned());
    }
    outcomes.sort();
    let store = Store::open(MemoryRoot::new(&root).unwrap()).unwrap();
    let entry_count = store.list().unwrap().entries().len();
    Competition {
        outcomes,
        processes,
        entry_count,
    }
}

fn spawn_worker(
    executable: &Path,
    root: &Path,
    go: &Path,
    ready: &Path,
    output: &Path,
    variant: &str,
) -> Child {
    Command::new(executable)
        .arg("--exact")
        .arg("concurrency_worker")
        .arg("--nocapture")
        .env("AGENT_MEMORY_ROOT", root)
        .env("AGENT_MEMORY_CONCURRENCY_GO", go)
        .env("AGENT_MEMORY_CONCURRENCY_READY", ready)
        .env("AGENT_MEMORY_CONCURRENCY_OUTPUT", output)
        .env("AGENT_MEMORY_CONCURRENCY_VARIANT", variant)
        .spawn()
        .unwrap()
}

fn wait_until_ready(paths: &[PathBuf]) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while paths.iter().any(|path| !path.exists()) {
        assert!(Instant::now() < deadline, "workers did not become ready");
        std::thread::sleep(Duration::from_millis(5));
    }
}
