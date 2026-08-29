#![allow(dead_code)]

use agent_memory::{
    Clock, Index, MemoryEntry, MemoryRoot, OracleEnvironment, ProjectKey, SearchRequest,
    SearchSelection, SourceKind, SourceResolution, SourceResolver, Store, UtcTimestamp,
    parse_entry, parse_utc_timestamp, resolve_project, search,
};
use std::collections::VecDeque;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[allow(dead_code)]
#[path = "../support/memory.rs"]
mod process;

pub use process::{FakeProcessRunner, FakeResponse};

pub const QUERY: &str = "durable memory";

#[derive(Clone)]
pub struct FixedClock {
    timestamp: UtcTimestamp,
}

impl FixedClock {
    pub fn at(timestamp: &str) -> Self {
        Self {
            timestamp: parse_utc_timestamp(timestamp).unwrap(),
        }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> UtcTimestamp {
        self.timestamp.clone()
    }
}

pub struct FakeResolver {
    responses: Mutex<VecDeque<SourceResolution>>,
    calls: Mutex<Vec<(SourceKind, String)>>,
}

impl FakeResolver {
    pub fn with_responses(responses: impl IntoIterator<Item = SourceResolution>) -> Self {
        Self {
            responses: Mutex::new(responses.into_iter().collect()),
            calls: Mutex::new(Vec::new()),
        }
    }

    pub fn calls(&self) -> Vec<(SourceKind, String)> {
        self.calls.lock().unwrap().clone()
    }
}

impl SourceResolver for FakeResolver {
    fn resolve(&self, source: &agent_memory::EntrySource) -> SourceResolution {
        self.calls
            .lock()
            .unwrap()
            .push((source.kind(), source.locator().to_owned()));
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected source resolution")
    }
}

#[derive(Clone, Copy)]
pub struct SourceFixture<'a> {
    pub kind: &'a str,
    pub locator: &'a str,
    pub fingerprint: char,
}

pub fn fingerprint(character: char) -> String {
    format!("sha256:{}", character.to_string().repeat(64))
}

pub fn valid(character: char) -> SourceResolution {
    SourceResolution::Fingerprint(fingerprint(character))
}

pub fn entry(id_character: char, kind: &str, sources: &[SourceFixture<'_>]) -> MemoryEntry {
    parse_entry(&entry_yaml(id_character, kind, sources)).unwrap()
}

pub fn entry_yaml(id_character: char, kind: &str, sources: &[SourceFixture<'_>]) -> Vec<u8> {
    let automated = if sources.iter().all(|source| source.kind == "user-decision") {
        ""
    } else {
        "  automated:\n    kind: source-fingerprint\n    expected: all-proof-sources-unchanged\n"
    };
    let sources = sources
        .iter()
        .map(|source| {
            format!(
                "    - kind: {}\n      locator: {}\n      fingerprint: {}\n",
                source.kind,
                serde_json::to_string(source.locator).unwrap(),
                fingerprint(source.fingerprint)
            )
        })
        .collect::<String>();
    format!(
        "schema_version: 1\nid: mem_{}\nkind: {kind}\nstatus: active\nstatement: Durable memory statement {id_character}.\nscope:\n  type: user\nretrieval_terms:\n  - durable memory\nproof:\n  summary: Durable proof summary.\n  sources:\n{sources}  established_at: 2026-08-28T00:00:00Z\noracle:\n{automated}  human_fallback:\n    question: Does the proof still establish this memory?\n    valid_when: The proof remains observable.\n  outcomes:\n    valid: The memory remains established.\n    invalidated: The proof no longer establishes the memory.\ncreated_at: 2026-08-28T00:00:00Z\n",
        id_character.to_string().repeat(24)
    )
    .into_bytes()
}

pub fn memory_root(path: &Path) -> MemoryRoot {
    MemoryRoot::new(path).unwrap()
}

pub fn open_store(directory: &Path) -> (PathBuf, Store) {
    let root = directory.join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    (root, store)
}

pub fn write_user_entry(root: &Path, id_character: char, yaml: &[u8]) -> PathBuf {
    let path = root.join(format!(
        "entries/user/mem_{}.yaml",
        id_character.to_string().repeat(24)
    ));
    fs::write(&path, yaml).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    path
}

pub fn project_key(directory: &Path) -> ProjectKey {
    let common = directory.join("project.git");
    fs::create_dir(&common).unwrap();
    let runner = FakeProcessRunner::with_responses([FakeResponse::success(format!(
        "{}\n",
        common.display()
    ))]);
    resolve_project(directory, &runner).unwrap().key().clone()
}

pub fn select(store: &Store, project_key: &ProjectKey, limit: usize) -> SearchSelection {
    let index = Index::load_or_rebuild(store).unwrap().index;
    search(
        &index,
        SearchRequest {
            query: QUERY,
            project_key,
            include_user: true,
            limit,
        },
    )
}

pub fn environment() -> OracleEnvironment {
    OracleEnvironment::new("macos", "aarch64")
}

pub fn cache_json(root: &Path) -> serde_json::Value {
    serde_json::from_slice(&fs::read(root.join("oracle-cache.json")).unwrap()).unwrap()
}
