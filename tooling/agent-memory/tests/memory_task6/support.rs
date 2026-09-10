use agent_memory::{
    Clock, Index, MemoryEntry, MemoryRoot, OracleEnvironment, ProjectKey, SearchRequest,
    SearchSelection, SourceKind, SourceResolution, SourceResolver, Store, UtcTimestamp,
    parse_entry, parse_utc_timestamp, resolve_project, search,
};
use sha2::{Digest, Sha256};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub use crate::memory_support::{FakeProcessRunner, FakeResponse};

pub type FixtureResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub const QUERY: &str = "durable memory";

#[derive(Clone)]
pub struct FixedClock {
    timestamp: UtcTimestamp,
}

impl FixedClock {
    pub fn at(timestamp: &str) -> Result<Self, agent_memory::MemoryError> {
        Ok(Self {
            timestamp: parse_utc_timestamp(timestamp)?,
        })
    }
}

impl Clock for FixedClock {
    fn now(&self) -> UtcTimestamp {
        self.timestamp.clone()
    }
}

pub struct FakeResolver {
    responses: RefCell<VecDeque<SourceResolution>>,
    calls: RefCell<Vec<(SourceKind, String)>>,
    exhausted: Cell<bool>,
}

impl FakeResolver {
    pub fn with_responses(responses: impl IntoIterator<Item = SourceResolution>) -> Self {
        Self {
            responses: RefCell::new(responses.into_iter().collect()),
            calls: RefCell::new(Vec::new()),
            exhausted: Cell::new(false),
        }
    }

    pub fn calls(&self) -> Vec<(SourceKind, String)> {
        self.calls.borrow().clone()
    }

    pub fn checked<T>(
        &self,
        operation: impl FnOnce(&Self) -> FixtureResult<T>,
    ) -> FixtureResult<T> {
        let result = operation(self);
        if self.exhausted.get() {
            return Err("unexpected source resolution exhausted the fixture".into());
        }
        result
    }
}

impl SourceResolver for FakeResolver {
    fn resolve(&self, source: &agent_memory::EntrySource) -> SourceResolution {
        self.calls
            .borrow_mut()
            .push((source.kind(), source.locator().to_owned()));
        self.responses.borrow_mut().pop_front().unwrap_or_else(|| {
            self.exhausted.set(true);
            SourceResolution::Unavailable
        })
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

pub fn entry(
    id_character: char,
    kind: &str,
    sources: &[SourceFixture<'_>],
) -> FixtureResult<MemoryEntry> {
    Ok(parse_entry(&entry_yaml(id_character, kind, sources)?)?)
}

pub fn entry_yaml(
    id_character: char,
    kind: &str,
    sources: &[SourceFixture<'_>],
) -> FixtureResult<Vec<u8>> {
    let automated = if sources.iter().all(|source| source.kind == "user-decision") {
        ""
    } else {
        "  automated:\n    kind: source-fingerprint\n    expected: all-proof-sources-unchanged\n"
    };
    let mut source_yaml = String::new();
    for source in sources {
        write!(
            &mut source_yaml,
            "    - kind: {}\n      locator: {}\n      fingerprint: {}\n",
            source.kind,
            serde_json::Value::String(source.locator.to_owned()),
            fingerprint(source.fingerprint)
        )?;
    }
    let statement = format!("Durable memory statement {id_character}.");
    let id = canonical_entry_id(kind, "user", &statement);
    Ok(format!(
        "schema_version: 1\nid: {id}\nkind: {kind}\nstatus: active\nstatement: {statement}\nscope:\n  type: user\nretrieval_terms:\n  - durable memory\nproof:\n  summary: Durable proof summary.\n  sources:\n{source_yaml}  established_at: 2026-08-28T00:00:00Z\noracle:\n{automated}  human_fallback:\n    question: Does the proof still establish this memory?\n    valid_when: The proof remains observable.\n  outcomes:\n    valid: The memory remains established.\n    invalidated: The proof no longer establishes the memory.\ncreated_at: 2026-08-28T00:00:00Z\n"
    )
    .into_bytes())
}

pub fn project_entry_yaml(
    id_character: char,
    kind: &str,
    key: &ProjectKey,
    sources: &[SourceFixture<'_>],
) -> FixtureResult<Vec<u8>> {
    let statement = format!("Durable memory statement {id_character}.");
    let user_id = canonical_entry_id(kind, "user", &statement);
    let project_id = canonical_entry_id(kind, key.as_str(), &statement);
    Ok(String::from_utf8(entry_yaml(id_character, kind, sources)?)?
        .replace(&format!("id: {user_id}"), &format!("id: {project_id}"))
        .replace(
            "scope:\n  type: user\n",
            &format!("scope:\n  type: project\n  key: {}\n", key.as_str()),
        )
        .into_bytes())
}

pub fn memory_root(path: &Path) -> FixtureResult<MemoryRoot> {
    Ok(MemoryRoot::new(path)?)
}

pub fn open_store(directory: &Path) -> FixtureResult<(PathBuf, Store)> {
    let root = directory.join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    Ok((root, store))
}

pub fn write_user_entry(root: &Path, id_character: char, yaml: &[u8]) -> FixtureResult<PathBuf> {
    let id = yaml_entry_id(yaml)?;
    assert!(String::from_utf8_lossy(yaml).contains(id_character));
    let path = root.join(format!("entries/user/{id}.yaml"));
    fs::write(&path, yaml)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    Ok(path)
}

pub fn write_project_entry(
    root: &Path,
    key: &ProjectKey,
    id_character: char,
    yaml: &[u8],
) -> FixtureResult<PathBuf> {
    let directory = root.join("entries/project").join(key.as_str());
    fs::create_dir_all(&directory)?;
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))?;
    let id = yaml_entry_id(yaml)?;
    assert!(String::from_utf8_lossy(yaml).contains(id_character));
    let path = directory.join(format!("{id}.yaml"));
    fs::write(&path, yaml)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    Ok(path)
}

pub fn user_entry_id(id_character: char, kind: &str) -> String {
    canonical_entry_id(
        kind,
        "user",
        &format!("Durable memory statement {id_character}."),
    )
}

fn canonical_entry_id(kind: &str, scope: &str, statement: &str) -> String {
    let preimage = serde_json::Value::Array(vec![
        1_u8.into(),
        kind.into(),
        scope.into(),
        statement.into(),
    ])
    .to_string();
    let digest = format!("{:x}", Sha256::digest(preimage));
    format!("mem_{}", digest.chars().take(24).collect::<String>())
}

fn yaml_entry_id(yaml: &[u8]) -> FixtureResult<String> {
    let document: serde_yaml_ng::Value = serde_yaml_ng::from_slice(yaml)?;
    Ok(document
        .get("id")
        .and_then(serde_yaml_ng::Value::as_str)
        .ok_or("missing fixture entry id")?
        .to_owned())
}

pub fn project_key(directory: &Path) -> FixtureResult<ProjectKey> {
    let common = directory.join("project.git");
    fs::create_dir(&common)?;
    let runner = FakeProcessRunner::with_responses([FakeResponse::success(format!(
        "{}\n",
        common.display()
    ))]);
    Ok(resolve_project(directory, &runner)?.key().clone())
}

pub fn select(
    store: &Store,
    project_key: &ProjectKey,
    limit: usize,
) -> FixtureResult<SearchSelection> {
    let index = Index::load_or_rebuild(store)?.index;
    Ok(search(
        &index,
        SearchRequest {
            query: QUERY,
            project_key,
            include_user: true,
            limit,
        },
    ))
}

pub fn environment() -> OracleEnvironment {
    OracleEnvironment::new("macos", "aarch64")
}

pub fn cache_json(root: &Path) -> FixtureResult<serde_json::Value> {
    Ok(serde_json::from_slice(&fs::read(
        root.join("oracle-cache.json"),
    )?)?)
}

#[test]
fn unexpected_source_requests_fail_fixture_verification() -> FixtureResult<()> {
    let directory = tempfile::tempdir()?;
    let (_, store) = open_store(directory.path())?;
    let entry = entry(
        'a',
        "invariant",
        &[SourceFixture {
            kind: "local-file",
            locator: "/tmp/fixture-proof",
            fingerprint: 'a',
        }],
    )?;
    let clock = FixedClock::at("2026-08-28T00:00:00Z")?;
    let resolver = FakeResolver::with_responses([]);

    let result = resolver.checked(|resolver| {
        Ok(agent_memory::evaluate_oracle(
            &entry,
            &agent_memory::OracleContext::new(&store, &clock, resolver, environment()),
        ))
    });

    assert!(result.is_err());
    Ok(())
}
